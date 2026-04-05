//! Filesystem Adapter Tests — Integration and security verification
//!
//! @docs ARCHITECTURE:Networking

#[cfg(test)]
mod tests {
    use crate::adapter::filesystem::FilesystemAdapter;

    #[tokio::test]
    async fn filesystem_write_and_read_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let adapter = FilesystemAdapter::new(tmp.path().to_path_buf());

        adapter.write_file("test.txt", "hello world").await.unwrap();
        let content = adapter.read_file("test.txt").await.unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn filesystem_list_files_returns_sorted() {
        let tmp = tempfile::tempdir().unwrap();
        let adapter = FilesystemAdapter::new(tmp.path().to_path_buf());

        adapter.write_file("charlie.txt", "c").await.unwrap();
        adapter.write_file("alpha.txt", "a").await.unwrap();
        adapter.write_file("bravo.txt", "b").await.unwrap();

        let files = adapter.list_files(".").await.unwrap();
        assert_eq!(files, vec!["alpha.txt", "bravo.txt", "charlie.txt"]);
    }

    #[tokio::test]
    async fn filesystem_delete_file_removes_it() {
        let tmp = tempfile::tempdir().unwrap();
        let adapter = FilesystemAdapter::new(tmp.path().to_path_buf());

        adapter.write_file("doomed.txt", "bye").await.unwrap();
        assert!(adapter.read_file("doomed.txt").await.is_ok());

        adapter.delete_file("doomed.txt").await.unwrap();
        assert!(adapter.read_file("doomed.txt").await.is_err());
    }

    #[tokio::test]
    async fn filesystem_rejects_path_traversal_dotdot() {
        let tmp = tempfile::tempdir().unwrap();
        let adapter = FilesystemAdapter::new(tmp.path().to_path_buf());

        let result = adapter.read_file("../../../etc/passwd").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SECURITY FAULT"));
    }

    #[tokio::test]
    async fn filesystem_rejects_absolute_path_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let adapter = FilesystemAdapter::new(tmp.path().to_path_buf());

        // Absolute paths get stripped of their root, so /etc/passwd
        // becomes etc/passwd under the sandbox — which won't exist.
        // The key thing is it doesn't escape.
        let result = adapter.read_file("/etc/passwd").await;
        // Should fail because "etc/passwd" doesn't exist in the sandbox
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn filesystem_nested_directory_creation() {
        let tmp = tempfile::tempdir().unwrap();
        let adapter = FilesystemAdapter::new(tmp.path().to_path_buf());

        adapter
            .write_file("deep/nested/dir/file.md", "# Title")
            .await
            .unwrap();
        let content = adapter.read_file("deep/nested/dir/file.md").await.unwrap();
        assert_eq!(content, "# Title");
    }

    #[tokio::test]
    async fn filesystem_list_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let adapter = FilesystemAdapter::new(tmp.path().to_path_buf());

        let files = adapter.list_files("nonexistent").await.unwrap();
        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn filesystem_list_shows_directories_with_suffix() {
        let tmp = tempfile::tempdir().unwrap();
        let adapter = FilesystemAdapter::new(tmp.path().to_path_buf());

        adapter.write_file("subdir/inner.txt", "x").await.unwrap();
        adapter.write_file("file.txt", "y").await.unwrap();

        let files = adapter.list_files(".").await.unwrap();
        assert!(files.contains(&"subdir/".to_string()));
        assert!(files.contains(&"file.txt".to_string()));
    }
}
