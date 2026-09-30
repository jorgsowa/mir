//! `FsSourceProvider` serves prefetched text once, then goes back to disk.

use mir_analyzer::{FsSourceProvider, SourceProvider};

fn path_str(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

#[test]
fn prefetched_text_is_served_once_then_disk_is_read() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("A.php");
    std::fs::write(&file, "<?php // v1").unwrap();
    let path = path_str(&file);

    let provider = FsSourceProvider::new();
    provider.prefetch(std::slice::from_ref(&path));
    std::fs::write(&file, "<?php // v2").unwrap();

    assert_eq!(provider.read(&path).as_deref(), Some("<?php // v1"));
    assert_eq!(provider.read(&path).as_deref(), Some("<?php // v2"));
}

#[test]
fn clear_prefetch_drops_unread_text() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("A.php");
    std::fs::write(&file, "<?php // v1").unwrap();
    let path = path_str(&file);

    let provider = FsSourceProvider::new();
    provider.prefetch(std::slice::from_ref(&path));
    provider.clear_prefetch();
    std::fs::write(&file, "<?php // v2").unwrap();

    assert_eq!(provider.read(&path).as_deref(), Some("<?php // v2"));
}

#[test]
fn prefetching_a_missing_path_reads_as_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_str(&dir.path().join("Missing.php"));

    let provider = FsSourceProvider::new();
    provider.prefetch(std::slice::from_ref(&path));

    assert_eq!(provider.read(&path), None);
}
