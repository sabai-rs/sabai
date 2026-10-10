use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use pollster::block_on;
use sabai::contracts::{BoxFuture, Clock, FileMetadata, StorageDriver};
use sabai::{Bytes, Result};

use super::FrozenClock;

/// An in-memory disk that implements only the required methods, so the tests below
/// also cover the default `exists`, `copy` and `rename`.
struct MemoryDisk {
    clock: Arc<dyn Clock>,
    files: Mutex<BTreeMap<String, (Bytes, SystemTime)>>,
}

impl StorageDriver for MemoryDisk {
    fn read<'a>(&'a self, path: &'a str) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            Ok(self
                .files
                .lock()
                .unwrap()
                .get(path)
                .map(|(bytes, _)| bytes.clone()))
        })
    }

    fn write<'a>(&'a self, path: &'a str, contents: Bytes) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let written_at = self.clock.now();
            self.files
                .lock()
                .unwrap()
                .insert(path.to_owned(), (contents, written_at));
            Ok(())
        })
    }

    fn delete<'a>(&'a self, path: &'a str) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move { Ok(self.files.lock().unwrap().remove(path).is_some()) })
    }

    fn metadata<'a>(&'a self, path: &'a str) -> BoxFuture<'a, Result<Option<FileMetadata>>> {
        Box::pin(async move {
            let files = self.files.lock().unwrap();
            let metadata = files
                .get(path)
                .map(|(bytes, at)| FileMetadata::new(bytes.len() as u64, *at));
            Ok(metadata)
        })
    }

    fn list<'a>(&'a self, directory: &'a str) -> BoxFuture<'a, Result<Vec<String>>> {
        Box::pin(async move {
            let files = self.files.lock().unwrap();
            Ok(files
                .keys()
                .filter(|path| path.starts_with(directory))
                .cloned()
                .collect())
        })
    }
}

fn disk() -> (MemoryDisk, Arc<FrozenClock>) {
    let clock = Arc::new(FrozenClock::at_epoch());
    let disk = MemoryDisk {
        clock: clock.clone(),
        files: Mutex::default(),
    };
    (disk, clock)
}

#[test]
fn a_written_file_can_be_read_back_with_its_metadata() {
    let (disk, clock) = disk();
    clock.travel(Duration::from_secs(10));

    block_on(disk.write("avatars/1.png", Bytes::from("png"))).unwrap();

    assert_eq!(
        block_on(disk.read("avatars/1.png")).unwrap(),
        Some(Bytes::from("png"))
    );
    let metadata = block_on(disk.metadata("avatars/1.png")).unwrap().unwrap();
    assert_eq!(metadata.size, 3);
    assert_eq!(
        metadata.last_modified,
        SystemTime::UNIX_EPOCH + Duration::from_secs(10)
    );
}

#[test]
fn a_missing_file_is_none_not_an_error() {
    let (disk, _clock) = disk();

    assert_eq!(block_on(disk.read("missing.txt")).unwrap(), None);
    assert!(!block_on(disk.exists("missing.txt")).unwrap());
    assert!(!block_on(disk.delete("missing.txt")).unwrap());
}

#[test]
fn list_returns_every_file_under_a_directory() {
    let (disk, _clock) = disk();
    for path in ["avatars/1.png", "avatars/2023/2.png", "docs/terms.pdf"] {
        block_on(disk.write(path, Bytes::new())).unwrap();
    }

    assert_eq!(
        block_on(disk.list("avatars/")).unwrap(),
        ["avatars/1.png", "avatars/2023/2.png"]
    );
    assert_eq!(block_on(disk.list("")).unwrap().len(), 3);
}

#[test]
fn default_copy_and_rename_work_for_any_driver() {
    let (disk, _clock) = disk();
    block_on(disk.write("draft.md", Bytes::from("hello"))).unwrap();

    block_on(disk.copy("draft.md", "backup.md")).unwrap();
    block_on(disk.rename("draft.md", "post.md")).unwrap();

    assert!(block_on(disk.exists("backup.md")).unwrap());
    assert!(block_on(disk.exists("post.md")).unwrap());
    assert!(!block_on(disk.exists("draft.md")).unwrap());
}

#[test]
fn copying_a_missing_file_names_it() {
    let (disk, _clock) = disk();

    let error = block_on(disk.copy("ghost.txt", "copy.txt")).unwrap_err();

    assert_eq!(error.to_string(), "file `ghost.txt` does not exist");
}
