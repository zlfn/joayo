use std::{process::Stdio, time::Duration};

use bytes::Bytes;
use libavif::is_avif;
use tokio::{fs, io::{AsyncReadExt, AsyncWriteExt}, process::Command, time::timeout};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{FFMPEG_PATH, IMAGE_PATH};

#[derive(Debug)]
pub enum ImageServiceError {
    TranscodeFailed,
    TranscodeTimeout,
    InternalServerError,
}

pub async fn convert_to_avif
(ffmpeg_timeout: u64, image_bytes: &Bytes) -> Result<Vec<u8>, ImageServiceError> {

    let temp_name = Uuid::now_v7();

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("{}/{}", IMAGE_PATH, temp_name))
        .await
        .unwrap();

    file.write_all(&image_bytes).await.unwrap();

    let ffmpeg_thread = Command::new(FFMPEG_PATH)
        .args(["-i", format!("./images/{}", temp_name).as_str()])
        .args(["-y"])
        .args(["-crf", "40"])
        .args(["-cpu-used", "8"])
        .arg(format!("{}/out_{}.avif", IMAGE_PATH, temp_name).as_str())
        .kill_on_drop(true)
        .stderr(Stdio::null())
        .status();

    let ffmpeg_result = timeout(Duration::from_secs(ffmpeg_timeout), ffmpeg_thread).await;

    if let Err(err) = fs::remove_file(format!("{}/{}", IMAGE_PATH, temp_name)).await {
        error!("Removing temp image failed: {}", err);
    };

    if let Err(err) = ffmpeg_result {
        warn!("Avif encoding timeout: {}", err);
        if let Err(err) = fs::remove_file(format!("{}/out_{}.avif", IMAGE_PATH, temp_name)).await {
            error!("Removing output image failed: {}", err);
        }
        return Result::Err(ImageServiceError::TranscodeTimeout);
    }

    let avif = fs::OpenOptions::new()
        .read(true)
        .open(format!("{}/out_{}.avif", IMAGE_PATH, temp_name))
        .await;

    let mut avif = match avif {
        Ok(avif) => avif,
        Err(err) => {
            warn!("Transcode failed: {}", err);
            return Result::Err(ImageServiceError::TranscodeFailed);
        }
    };

    let mut buf: Vec<u8> = Vec::new();
    if let Err(err) = avif.read_to_end(&mut buf).await {
        error!("Fail to load avif: {}", err);
    }

    if let Err(err) = fs::remove_file(format!("{}/out_{}.avif", IMAGE_PATH, temp_name)).await {
        error!("Removing output image failed: {}", err);
    }

    if !is_avif(buf.as_ref()) {
        warn!("FFMPEG failed to encode valid avif");
        return Result::Err(ImageServiceError::TranscodeFailed);
    }

    Result::Ok(buf)
}
