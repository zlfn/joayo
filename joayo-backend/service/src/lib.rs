use std::{sync::Arc, time::Duration};

use bytes::Bytes;
use encode::convert_to_avif;
use tokio::{sync::watch, task};
use tracing::{error, info};

mod encode;

pub const IMAGE_PATH: &str = "./images";
pub const FFMPEG_PATH: &str = "./images/bin/ffmpeg";

pub async fn queue_executor(
    mut shutdown_rx: watch::Receiver<()>,
    encode_rx: crossbeam::channel::Receiver<Arc<Bytes>>
) {
    let image1 = task::spawn(image_queue_executor("IMAGE1", shutdown_rx.clone(), encode_rx.clone()));
    let image2 = task::spawn(image_queue_executor("IMAGE2", shutdown_rx.clone(), encode_rx.clone()));
    let image3 = task::spawn(image_queue_executor("IMAGE3", shutdown_rx.clone(), encode_rx.clone()));

    shutdown_rx.changed().await.unwrap();
    info!("Shutting down services...");
    
    image1.await.unwrap();
    image2.await.unwrap();
    image3.await.unwrap();
}

async fn image_queue_executor(
    name: &str,
    shutdown_rx: watch::Receiver<()>,
    encode_rx: crossbeam::channel::Receiver<Arc<Bytes>>
) {
    loop {
        if shutdown_rx.has_changed().unwrap() {
            info!("Shutting down service {}...", name);

            if !encode_rx.is_empty() {
                info!("{}: Processing remaining requests...", name);
            }

            while !encode_rx.is_empty() {
                receive_image(name, &encode_rx).await;
            }

            break;
        }

        receive_image(name, &encode_rx).await;
    }
}

async fn receive_image(name: &str, encode_rx: &crossbeam::channel::Receiver<Arc<Bytes>>) {
    let image_bytes = encode_rx.recv_timeout(Duration::from_secs(3));
    let image_bytes = match image_bytes {
        Ok(image_bytes) => image_bytes,
        Err(_) => return,
    };

    let image_bytes = Arc::clone(&image_bytes);

    info!("{}: Received image encoding request: {}KB", name, (*image_bytes).len()/1024);

    let avif_bytes = convert_to_avif(10, &*image_bytes).await;
    let avif_bytes = match avif_bytes {
        Ok(avif_bytes) => avif_bytes,
        Err(err) => {
            error!("{}: Encoding failed: {:?}", name, err);

            //TODO: Error handling logic.

            return;
        }
    };

    info!("{}: Encoding completed: {}KB", name, (*avif_bytes).len()/1024);

    //TODO: AVIF upload logic.

    return;
}
