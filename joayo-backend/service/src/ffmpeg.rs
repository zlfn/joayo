use std::{process::Stdio, time::Duration};

use bytes::Bytes;
use libavif::is_avif;
use tokio::{fs, io::{self, AsyncReadExt, AsyncWriteExt}, process::Command, time::timeout};
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Debug)]
pub enum ImageServiceError {
    TranscodeFailed,
    TranscodeTimeout,
}

pub struct FFmpegConverter {
    ffmpeg_path: String,
    image_path: String,
    timeout: Duration,
    crf: i8,
    threads: u32,
}

impl FFmpegConverter {
    pub fn builder() -> FFmpegConverterBuilder {
        FFmpegConverterBuilder::new()
    }

    pub async fn convert
    (self, image_bytes: &Bytes) -> Result<Vec<u8>, ImageServiceError> {

        let temp_name = Uuid::now_v7();

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(format!("{}/{}", self.image_path, temp_name))
            .await
            .unwrap();

        file.write_all(&image_bytes).await.unwrap();

        let ffmpeg_thread = Command::new(self.ffmpeg_path)
            .args(["-i", format!("{}/{}", self.image_path, temp_name).as_str()])
            .args(["-y"])
            .args(["-crf", &self.crf.to_string()])
            .args(["-threads", &self.threads.to_string()])
            .args(["-cpu-used", "8"])
            .args(["-frames:v", "1"])
            .arg(format!("{}/out_{}.avif", self.image_path, temp_name).as_str())
            .kill_on_drop(true)
            .stderr(Stdio::null())
            .status();

        let ffmpeg_result = timeout(self.timeout, ffmpeg_thread).await;

        if let Err(err) = fs::remove_file(format!("{}/{}", self.image_path, temp_name)).await {
            error!("Removing temp image failed: {}", err);
        };

        if let Err(err) = ffmpeg_result {
            warn!("Avif encoding timeout: {}", err);
            if let Err(err) = fs::remove_file(format!("{}/out_{}.avif", self.image_path, temp_name)).await {
                error!("Removing output image failed: {}", err);
            }
            return Result::Err(ImageServiceError::TranscodeTimeout);
        }

        let avif = fs::OpenOptions::new()
            .read(true)
            .open(format!("{}/out_{}.avif", self.image_path, temp_name))
            .await;

        let mut avif = match avif {
            Ok(avif) => avif,
            Err(err) => {
                warn!("Transcode unavailable: {}", err);
                return Result::Err(ImageServiceError::TranscodeFailed);
            }
        };

        let mut buf: Vec<u8> = Vec::new();
        if let Err(err) = avif.read_to_end(&mut buf).await {
            error!("Fail to load avif: {}", err);
        }

        if let Err(err) = fs::remove_file(format!("{}/out_{}.avif", self.image_path, temp_name)).await {
            error!("Removing output image failed: {}", err);
        }

        if !is_avif(buf.as_ref()) {
            error!("FFMPEG failed to encode valid avif");
            return Result::Err(ImageServiceError::TranscodeFailed);
        }

        Result::Ok(buf)
    }
}

pub struct FFmpegConverterBuilder {
    timeout: Duration,
    crf: i8,
    threads: u32,
    ffmpeg_path: String,
    image_path: String,
}

impl FFmpegConverterBuilder {
    pub fn new() -> FFmpegConverterBuilder {
        FFmpegConverterBuilder {
            timeout: Duration::from_secs(30),
            crf: 30,
            threads: 1,
            ffmpeg_path: "./images/bin/ffmpeg".to_string(),
            image_path: "./images".to_string(),
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn threads(mut self, threads: u32) -> Self {
        self.threads = threads;
        self
    }

    pub fn ffmpeg_path(mut self, path: String) -> Self {
        self.ffmpeg_path = path;
        self
    }

    pub fn image_path(mut self, path: String) -> Self {
        self.image_path = path;
        self
    }

    pub fn crf(mut self, crf: i8) -> Result<Self, io::Error> {
        if crf < -1 || crf > 63 {
            return Err(io::Error::new(std::io::ErrorKind::InvalidData, "crf must be a value in -1~63"));
        }
        self.crf = crf;
        return Ok(self);
    }

    pub fn build<'a>(self) -> FFmpegConverter {
        FFmpegConverter {
            timeout: self.timeout,
            crf: self.crf,
            threads: self.threads,
            ffmpeg_path: self.ffmpeg_path,
            image_path: self.image_path
        }
    }
}

