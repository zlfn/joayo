use std::{fmt::Display, time::Duration};

use async_scoped::TokioScope;
use aws_sdk_s3::primitives::ByteStream;
use bytes::Bytes;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr};
use tokio::sync::watch;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::ffmpeg::FFmpegConverter;
use orm::entities::*;

pub struct ImageUploadRequest {
    pub joayo_id: Uuid,
    pub crf: i8,
    pub bytes: Bytes,
}

pub enum ImageUploadResult {
    Ok,
    EncodeFailed,
    UploadFailed,
    InternalServerError,
}

impl Display for ImageUploadResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "Ok"),
            Self::EncodeFailed => write!(f, "EncodeFailed"),
            Self::UploadFailed => write!(f, "UploadFailed"),
            Self::InternalServerError => write!(f, "InternalServerError"),
        }
    }
}

#[derive(Clone)]
pub struct ImageQueueExecutor {
    pub db: DatabaseConnection,
    pub ffmpeg_path: String,
    pub ffmpeg_timeout: Duration,
    pub ffmpeg_threads: u32,
    pub image_path: String,
    pub s3_name: String,
    pub s3_client: aws_sdk_s3::Client,
    pub s3_public_url: String,
    pub shutdown_rx: watch::Receiver<()>,
    pub encode_rx: crossbeam::channel::Receiver<ImageUploadRequest>
}

impl ImageQueueExecutor {
    pub async fn exec(&self, name: String) {
        while !self.shutdown_rx.has_changed().unwrap() {
            //https://github.com/awslabs/aws-sdk-rust/issues/611
            TokioScope::scope_and_block(
                |s| { s.spawn(self.process_image(&name)) }
            );
        }

        if !self.encode_rx.is_empty() {
            info!("{}: Processing remaining requests...", name);
        }

        while !self.encode_rx.is_empty() {
            TokioScope::scope_and_block(
                |s| { s.spawn(self.process_image(&name)) }
            );
        }
    }

    async fn process_image(&self, name: &str) {
        let request = self.encode_rx.recv_timeout(Duration::from_secs(3));
        let request = match request {
            Ok(request) => request,
            Err(_) => return,
        };

        let original_size = request.bytes.len();

        info!("{}: Received image encoding request: {}KB", name, original_size/1024);

        let ffmpeg = FFmpegConverter::builder()
            .timeout(self.ffmpeg_timeout)
            .threads(self.ffmpeg_threads)
            .image_path(self.image_path.clone())
            .ffmpeg_path(self.ffmpeg_path.clone())
            .crf(request.crf);
        
        let ffmpeg = match ffmpeg {
            Ok(ffmpeg) => ffmpeg.build(),
            Err(err) => {
                warn!("{}: FFmpeg configuration failed: {:?}", name, err);
                if let Err(err) = self.write_upload_result(
                    request.joayo_id, 
                    original_size as i32, 
                    None, None, 
                    ImageUploadResult::EncodeFailed
                ).await {
                    error!("{}: Failed to register upload result: {}", name, err);
                }
                return;
            }
        };

        let avif_bytes = ffmpeg.convert(&request.bytes).await;

        let avif_bytes = match avif_bytes {
            Ok(avif_bytes) => avif_bytes,
            Err(err) => {
                warn!("{}: Encoding failed: {:?}", name, err);
                if let Err(err) = self.write_upload_result(
                    request.joayo_id, 
                    original_size as i32, 
                    None, None, 
                    ImageUploadResult::EncodeFailed
                ).await {
                    error!("{}: Failed to register upload result: {}", name, err);
                }
                return;
            }
        };

        let file_size = (*avif_bytes).len();
        info!("{}: Encoding completed: {}KB", name, file_size/1024);

        let file_name = format!("{}.avif", Uuid::new_v4());

        let upload_result = &self.s3_client
            .put_object()
            .bucket(&self.s3_name)
            .key(&file_name)
            .body(ByteStream::from(avif_bytes))
            .send().await;

        if let Err(err) = upload_result {
            warn!("{}: S3 upload failed: {:?}", name, err);
            if let Err(err) = self.write_upload_result(
                request.joayo_id, 
                original_size as i32, 
                Some(file_size as i32),
                None, 
                ImageUploadResult::UploadFailed,
            ).await {
                error!("{}: Failed to register upload result: {}", name, err);
            }
            return;
        }

        if let Err(err) = self.write_upload_result(
            request.joayo_id, 
            original_size as i32, 
            Some(file_size as i32),
            Some(format!("{}/{}", self.s3_public_url, file_name)), 
            ImageUploadResult::Ok,
        ).await {
            error!("{}: Failed to register upload result: {}", name, err);
        }

        info!("{}: Upload completed: {}", name, file_name);

        return;
    }

    async fn write_upload_result(
        &self,
        joayo_id: Uuid, 
        original_size: i32,
        file_size: Option<i32>,
        object_url: Option<String>,
        result: ImageUploadResult
    ) -> Result<(), DbErr> {
        upload_result::ActiveModel {
            joayo_id: ActiveValue::Set(joayo_id),
            original_size: ActiveValue::Set(original_size),
            file_size: ActiveValue::Set(file_size),
            object_url: ActiveValue::Set(object_url),
            result: ActiveValue::Set(result.to_string())
        }.insert(&self.db).await?;

        Ok(())
    }
}
