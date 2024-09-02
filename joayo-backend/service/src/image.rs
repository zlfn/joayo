use std::{io, time::Duration};

use async_scoped::TokioScope;
use aws_sdk_s3::primitives::ByteStream;
use bytes::Bytes;
use tokio::sync::watch;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::ffmpeg::FFmpegConverter;

pub struct ImageUploadRequest {
    pub joayo_uuid: Uuid,
    pub crf: i8,
    pub bytes: Bytes,
}

#[derive(Clone)]
pub struct ImageQueueExecutor {
    ffmpeg_path: String,
    ffmpeg_timeout: Duration,
    ffmpeg_threads: u32,
    image_path: String,
    s3_name: String,
    s3_client: aws_sdk_s3::Client,
    shutdown_rx: watch::Receiver<()>,
    encode_rx: crossbeam::channel::Receiver<ImageUploadRequest>
}

impl ImageQueueExecutor {
    pub fn builder() -> ImageQueueExecutorBuilder {
        ImageQueueExecutorBuilder::new()
    }

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

        info!("{}: Received image encoding request: {}KB", name, request.bytes.len()/1024);

        let ffmpeg = FFmpegConverter::builder()
            .timeout(self.ffmpeg_timeout)
            .threads(self.ffmpeg_threads)
            .image_path(self.image_path.clone())
            .ffmpeg_path(self.ffmpeg_path.clone())
            .crf(request.crf);
        
        let ffmpeg = match ffmpeg {
            Ok(ffmpeg) => ffmpeg.build(),
            Err(err) => {
                error!("{}", err);

                //TODO: Error handling logic.

                return;
            }
        };

        let avif_bytes = ffmpeg.convert(&request.bytes).await;

        let avif_bytes = match avif_bytes {
            Ok(avif_bytes) => avif_bytes,
            Err(err) => {
                warn!("{}: Encoding failed: {:?}", name, err);

                //TODO: Error handling logic.

                return;
            }
        };

        info!("{}: Encoding completed: {}KB", name, (*avif_bytes).len()/1024);

        let file_name = format!("{}.avif", Uuid::now_v7());

        let upload_result = &self.s3_client
            .put_object()
            .bucket(&self.s3_name)
            .key(&file_name)
            .body(ByteStream::from(avif_bytes))
            .send().await;

        if let Err(err) = upload_result {
            error!("{:?}", err);

            //TODO: Error handling logic.

        }

        //TODO: Database update logic.

        info!("{}: Upload completed: {}", name, file_name);

        return;
    }
}

pub struct ImageQueueExecutorBuilder {
    name: String,
    image_path: String,
    ffmpeg_path: String,
    ffmpeg_timeout: Duration,
    ffmpeg_threads: u32,
    s3_name: String,
    s3_client: Option<aws_sdk_s3::Client>,
    shutdown_rx: Option<watch::Receiver<()>>,
    encode_rx: Option<crossbeam::channel::Receiver<ImageUploadRequest>>
}

impl ImageQueueExecutorBuilder {
    pub fn new() -> Self {
        Self {
            name: "IMAGE".to_string(),
            image_path: "./images".to_string(),
            ffmpeg_path: "./images/bin/ffmpeg".to_string(),
            ffmpeg_timeout: Duration::from_secs(30),
            ffmpeg_threads: 4,
            s3_name: "joayo-r2".to_string(),
            s3_client: None,
            shutdown_rx: None,
            encode_rx: None,
        }
    }

    pub fn build(&self) -> Result<ImageQueueExecutor, io::Error> {
        let s3_client = match &self.s3_client {
            Some(s3_client) => s3_client,
            None => return Result::Err(io::Error::new(io::ErrorKind::InvalidData, "s3_client is None"))
        };
        let shutdown_rx = match &self.shutdown_rx {
            Some(shutdown_rx) => shutdown_rx,
            None => return Result::Err(io::Error::new(io::ErrorKind::InvalidData, "shutdown_rx is None"))
        };
        let encode_rx = match &self.encode_rx {
            Some(encode_rx) => encode_rx,
            None => return Result::Err(io::Error::new(io::ErrorKind::InvalidData, "encode_rx is None"))
        };
        Ok(ImageQueueExecutor {
            image_path: self.image_path.clone(),
            ffmpeg_path: self.ffmpeg_path.clone(),
            ffmpeg_timeout: self.ffmpeg_timeout,
            ffmpeg_threads: self.ffmpeg_threads,
            s3_name: self.s3_name.clone(),
            s3_client: s3_client.clone(),
            shutdown_rx: shutdown_rx.clone(),
            encode_rx: encode_rx.clone()
        })
    }

    pub fn image_path(mut self, path: String) -> Self {
        self.image_path = path;
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn ffmpeg_path(mut self, path: String) -> Self {
        self.ffmpeg_path = path;
        self
    }

    pub fn ffmpeg_timeout(mut self, timeout: Duration) -> Self {
        self.ffmpeg_timeout = timeout;
        self
    }

    pub fn ffmpeg_threads(mut self, threads: u32) -> Self {
        self.ffmpeg_threads = threads;
        self
    }

    pub fn s3_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn s3_client(mut self, client: aws_sdk_s3::Client) -> Self {
        self.s3_client = Some(client);
        self
    }

    pub fn shutdown_rx(mut self, shutdown_rx: watch::Receiver<()>) -> Self {
        self.shutdown_rx = Some(shutdown_rx);
        self
    }

    pub fn encode_rx(mut self, encode_rx: crossbeam::channel::Receiver<ImageUploadRequest>) -> Self {
        self.encode_rx = Some(encode_rx);
        self
    }
}
