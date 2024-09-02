use async_scoped::{spawner::use_tokio::Tokio, TokioScope};
use image::{ImageQueueExecutor, ImageUploadRequest};
use s3::S3ClientCreator;
use config::Config;
use tokio::sync::watch;
use tracing::info;

pub mod image;
pub mod ffmpeg;
pub mod s3;

pub async fn queue_executor(
    configs: Config,
    mut shutdown_rx: watch::Receiver<()>,
    encode_rx: crossbeam::channel::Receiver<ImageUploadRequest>
) {
    let bucket_client = S3ClientCreator {
        access_key_id: configs.get_string("bucket.credentials.access-key-id").unwrap(),
        secret_access_key: configs.get_string("bucket.credentials.secret-access-key").unwrap(),
        endpoint: configs.get_string("bucket.endpoint").unwrap(),
    };

    let image_executor = ImageQueueExecutor::builder()
        .ffmpeg_path(configs.get_string("encoder.local.ffmpeg-path").unwrap())
        .image_path(configs.get_string("encoder.local.image-path").unwrap())
        .s3_name(configs.get_string("bucket.name").unwrap())
        .s3_client(bucket_client.create())
        .shutdown_rx(shutdown_rx.clone())
        .encode_rx(encode_rx.clone())
        .build()
        .unwrap();

    let mut scope = unsafe {
        TokioScope::<()>::create(Tokio)
    };

    for i in 0..configs.get_int("encoder.local.count").unwrap() {
        scope.spawn((&image_executor).exec(format!("IMAGE{}", i)));
    }

    shutdown_rx.changed().await.unwrap();
    info!("Shutting down services...");

    scope.collect().await;
}
