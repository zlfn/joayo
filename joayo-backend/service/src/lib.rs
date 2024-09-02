use std::time::Duration;

use aws_config::Region;
use aws_sdk_s3::{config::Credentials, primitives::ByteStream};
use config::Config;
use bytes::Bytes;
use ffmpeg::FFmpegConverter;
use tokio::{sync::watch, task::{self, JoinHandle}};
use tracing::{warn, info, error};
use uuid::Uuid;

mod ffmpeg;
mod aws;

pub async fn queue_executor(
    configs: Config,
    mut shutdown_rx: watch::Receiver<()>,
    encode_rx: crossbeam::channel::Receiver<Bytes>
) {
    let mut image_queues: Vec<JoinHandle<()>> = Vec::new();

    let aws_credentials = Credentials::new(
        configs.get_string("bucket.credentials.access-key-id").unwrap(),
        configs.get_string("bucket.credentials.secret-access-key").unwrap(),
        None, None,
        "credentials"
    );

    let aws_config = aws_sdk_s3::Config::builder()
        .credentials_provider(aws_credentials)
        .endpoint_url(configs.get_string("bucket.endpoint").unwrap())
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::v2024_03_28())
        .region(Region::from_static("auto"))
        .build();

    let aws_client = aws_sdk_s3::Client::from_conf(aws_config);

    for i in 0..configs.get_int("encoder.local.count").unwrap() {
        image_queues.push(
            task::spawn(
                image_queue_executor(
                    format!("IMAGE{}", i), 
                    configs.clone(), 
                    aws_client.clone(),
                    shutdown_rx.clone(), 
                    encode_rx.clone()
                )
            )
        );
    }

    shutdown_rx.changed().await.unwrap();
    info!("Shutting down services...");

    for encoder in image_queues {
        encoder.await.unwrap();
    }
}

async fn image_queue_executor(
    name: String,
    configs: Config,
    aws_client: aws_sdk_s3::Client,
    shutdown_rx: watch::Receiver<()>,
    encode_rx: crossbeam::channel::Receiver<Bytes>
) {
    loop {
        if shutdown_rx.has_changed().unwrap() {
            info!("Shutting down service {}...", name);

            if !encode_rx.is_empty() {
                info!("{}: Processing remaining requests...", name);
            }

            while !encode_rx.is_empty() {
                receive_image(&name, &configs, &aws_client, &encode_rx).await;
            }

            break;
        }

        receive_image(&name, &configs, &aws_client, &encode_rx).await;
    }
}

async fn receive_image(
    name: &str, 
    configs: &Config, 
    aws_client: &aws_sdk_s3::Client,
    encode_rx: &crossbeam::channel::Receiver<Bytes>
) {
    let image_bytes = encode_rx.recv_timeout(Duration::from_secs(3));
    let image_bytes = match image_bytes {
        Ok(image_bytes) => image_bytes,
        Err(_) => return,
    };

    info!("{}: Received image encoding request: {}KB", name, image_bytes.len()/1024);

    let ffmpeg = FFmpegConverter::builder()
        .timeout(Duration::from_secs(10))
        .threads(4)
        .image_path(configs.get_string("encoder.local.image-path").unwrap())
        .ffmpeg_path(configs.get_string("encoder.local.ffmpeg-path").unwrap())
        .crf(40).unwrap()
        .build();

    let avif_bytes = ffmpeg.convert(&image_bytes).await;

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

    let upload_result = aws_client
        .put_object()
        .bucket(configs.get_string("bucket.name").unwrap())
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
