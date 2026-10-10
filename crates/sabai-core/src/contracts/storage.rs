use std::error::Error as StdError;
use std::fmt;
use std::time::SystemTime;

use bytes::Bytes;

use super::BoxFuture;
use crate::{Error, Result};

/// Size and modification time of a stored file.
// `non_exhaustive` so fields such as a MIME type can be added without breaking drivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct FileMetadata {
    /// Size in bytes.
    pub size: u64,
    /// When the file was last written.
    pub last_modified: SystemTime,
}

impl FileMetadata {
    /// Metadata for a file of `size` bytes last written at `last_modified`.
    pub fn new(size: u64, last_modified: SystemTime) -> Self {
        Self {
            size,
            last_modified,
        }
    }
}

/// A storage driver such as the local disk or S3, like a Flysystem adapter. The app-facing
/// `Filesystem` (`Storage::disk("s3")`) is built on top of these methods.
///
/// Paths are relative and `/`-separated, such as `avatars/1.png`. Drivers must reject `..`
/// segments so a path from user input can never escape the disk root.
pub trait StorageDriver: Send + Sync {
    /// The file's contents, or `None` when it does not exist.
    fn read<'a>(&'a self, path: &'a str) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Writes `contents` to `path`, creating parent directories and replacing any existing file.
    fn write<'a>(&'a self, path: &'a str, contents: Bytes) -> BoxFuture<'a, Result<()>>;

    /// Removes the file; `true` when it was there.
    fn delete<'a>(&'a self, path: &'a str) -> BoxFuture<'a, Result<bool>>;

    /// The file's size and modification time in one call, or `None` when it does not exist.
    fn metadata<'a>(&'a self, path: &'a str) -> BoxFuture<'a, Result<Option<FileMetadata>>>;

    /// Every file under `directory`, recursively, as full paths; `""` lists the whole disk.
    fn list<'a>(&'a self, directory: &'a str) -> BoxFuture<'a, Result<Vec<String>>>;

    /// Whether the file exists.
    fn exists<'a>(&'a self, path: &'a str) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move { Ok(self.metadata(path).await?.is_some()) })
    }

    /// Copies a file. The default reads and writes it; drivers such as S3 can copy server-side.
    fn copy<'a>(&'a self, from: &'a str, to: &'a str) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let contents = self.read(from).await?.ok_or_else(|| file_not_found(from))?;
            self.write(to, contents).await
        })
    }

    /// Moves a file (Laravel's `move`, a keyword in Rust). The default copies, then deletes.
    fn rename<'a>(&'a self, from: &'a str, to: &'a str) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.copy(from, to).await?;
            self.delete(from).await?;
            Ok(())
        })
    }
}

#[derive(Debug)]
struct FileNotFound {
    path: String,
}

impl fmt::Display for FileNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file `{}` does not exist", self.path)
    }
}

impl StdError for FileNotFound {}

fn file_not_found(path: &str) -> Error {
    FileNotFound {
        path: path.to_owned(),
    }
    .into()
}
