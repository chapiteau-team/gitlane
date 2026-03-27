//! Filesystem helpers with consistent `FsError` mapping.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsError {
    #[error("expected directory at `{path}`")]
    ExpectedDirectory { path: PathBuf },
    #[error("failed to create directory `{path}`")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("expected file at `{path}`")]
    ExpectedFile { path: PathBuf },
    #[error("failed to read `{path}`")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write `{path}`")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// Ensure `path` is a directory, creating it (and parents) if missing.
pub(crate) fn ensure_directory(path: &Path) -> Result<(), FsError> {
    if path.is_dir() {
        return Ok(());
    }

    let path_buf = path.to_path_buf();

    if path.exists() {
        return Err(FsError::ExpectedDirectory { path: path_buf });
    }

    fs::create_dir_all(path).map_err(|source| FsError::CreateDirectory {
        path: path_buf,
        source,
    })
}

/// Ensure `path` points to an existing regular file.
///
/// Missing paths and non-file paths both return `FsError::ExpectedFile`.
pub(crate) fn ensure_file(path: &Path) -> Result<(), FsError> {
    if path.is_file() {
        return Ok(());
    }

    Err(FsError::ExpectedFile {
        path: path.to_path_buf(),
    })
}

/// Read a UTF-8 text file at `path`.
pub(crate) fn read_text_file(path: &Path) -> Result<String, FsError> {
    fs::read_to_string(path).map_err(|source| FsError::ReadFile {
        path: path.to_path_buf(),
        source,
    })
}

/// Write UTF-8 text content to `path`, replacing the file if it exists.
pub(crate) fn write_text_file(path: &Path, content: &str) -> Result<(), FsError> {
    fs::write(path, content).map_err(|source| FsError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    #[test]
    fn ensure_directory_creates_missing_path() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let nested = temp_dir.path().join("a").join("b");

        ensure_directory(&nested).expect("directory should be created");

        assert!(nested.is_dir());
    }

    #[test]
    fn ensure_directory_errors_when_path_is_file() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let file_path = temp_dir.path().join("data.txt");
        fs::write(&file_path, "data").expect("file should be created");

        let err = ensure_directory(&file_path).expect_err("file path should fail");

        assert!(matches!(err, FsError::ExpectedDirectory { .. }));
    }

    #[test]
    fn ensure_file_accepts_existing_file() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let file_path = temp_dir.path().join("config.toml");
        fs::write(&file_path, "name = \"gitlane\"\n").expect("file should be created");

        ensure_file(&file_path).expect("existing file should pass");
    }

    #[test]
    fn ensure_file_errors_when_path_is_directory() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let dir_path = temp_dir.path().join("config");
        fs::create_dir_all(&dir_path).expect("directory should be created");

        let err = ensure_file(&dir_path).expect_err("directory path should fail");

        assert!(matches!(err, FsError::ExpectedFile { .. }));
    }
}
