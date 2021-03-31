use crate::gen::challenger::{Locations,Batch};

use bytes::Bytes;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use prost::Message;

use thiserror::Error;

#[derive(Error,Debug)]
pub enum LoadError {
    #[error("Failed to open {filename}: {source}")]
    FileOpenError {
        filename: String,
        #[source]
        source: tokio::io::Error,
    },
    #[error("Failed to read_to_end {filename}: {source}")]
    FileReadError {
        filename: String,
        #[source]
        source: tokio::io::Error,
    },
    #[error("Failed to decode data of {filename}: {source}")]
    FileDecodeError {
        filename: String,
        #[source]
        source: prost::DecodeError,
    }
}

pub async fn load_locations(root: &str) -> Result<Locations, LoadError> {
    let filename = format!("{}/locations_dump.bin", &root);
    let mut f = File::open(&filename)
        .await
        .map_err(|source| LoadError::FileOpenError { filename: filename.clone(), source })?;
    let mut data = vec![];
    f.read_to_end(&mut data)
        .await
        .map_err(|source| LoadError::FileReadError { filename: filename.clone(), source })?;
    let b = Bytes::from(data);
    Message::decode(b).map_err(|source| LoadError::FileDecodeError { filename: filename, source })
}

pub async fn load_batch(root: &str, num: usize) -> Result<Batch, LoadError> {
    let dir = num/1000;
    let filename = format!("{}/messages/{}/batch_{}.bin", root, dir, num);
    load_batch_from(&filename).await
}

pub async fn load_batch_from(filename: &str) -> Result<Batch, LoadError> {
    let mut f = File::open(filename)
        .await
        .map_err(|source| LoadError::FileOpenError { filename: filename.to_owned(), source })?;
    let mut data = vec![];
    f.read_to_end(&mut data)
        .await
        .map_err(|source| LoadError::FileReadError { filename: filename.to_owned(), source })?;
    let b = Bytes::from(data);
    Batch::decode(b).map_err(|source| LoadError::FileDecodeError { filename: filename.to_owned(), source })
}
