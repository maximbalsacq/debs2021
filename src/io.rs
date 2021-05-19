use crate::gen::challenger::{Locations,Batch};

use bytes::Bytes;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use prost::Message;

use thiserror::Error;

/// A Loading Error, caused by an error when
/// - opening the file
/// - reading the file
/// - attempting to decode the file
#[derive(Error,Debug)]
pub enum LoadError {
    /// Returned when tokio::File::open() fails.
    #[error("Failed to open {filename}: {source}")]
    FileOpenError {
        /// The filename of the file which could not be opened.
        filename: String,
        /// The error returned by tokio::File::open().
        #[source]
        source: tokio::io::Error,
    },
    /// Returned when tokio::File::read_to_end fails.
    #[error("Failed to read_to_end {filename}: {source}")]
    FileReadError {
        /// The filename of the file which could not be read.
        filename: String,
        /// The error returned by tokio::File::read_to_end().
        #[source]
        source: tokio::io::Error,
    },
    /// Returned when the protobuf data contained
    /// in the file cannot be decoded.
    #[error("Failed to decode data of {filename}: {source}")]
    FileDecodeError {
        /// The filename of the file of which the contents
        /// could not be decrypted.
        filename: String,
        /// The error which occurred during decoding.
        #[source]
        source: prost::DecodeError,
    }
}

/// Loads the protobuf locations data from a file
/// named locations_dump.bin in the directory `root`.
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

/// Loads the batch of measurements num relative to the directory `root`.
pub async fn load_batch(root: &str, num: usize) -> Result<Batch, LoadError> {
    let dir = num/1000;
    let filename = format!("{}/messages/{}/batch_{}.bin", root, dir, num);
    load_batch_from(&filename).await
}

/// Loads a batch from the specified filename path.
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
