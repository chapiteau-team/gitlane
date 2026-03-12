use std::{
    fs, io,
    path::{Path, PathBuf},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsError {
    #[error("expected directory at `{path}`")]
    ExpectedDirectory { path: PathBuf },
    #[error("expected file at `{path}`")]
    ExpectedFile { path: PathBuf },
    #[error("failed to create directory `{path}`")]
    CreateDirectory {
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

pub fn ensure_directory(path: &Path) -> Result<(), FsError> {
    if path.exists() {
        if path.is_dir() {
            return Ok(());
        }

        return Err(FsError::ExpectedDirectory {
            path: path.to_path_buf(),
        });
    }

    fs::create_dir_all(path).map_err(|source| FsError::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}

pub fn write_file_if_missing(path: &Path, content: &str) -> Result<(), FsError> {
    if path.exists() {
        if path.is_file() {
            return Ok(());
        }

        return Err(FsError::ExpectedFile {
            path: path.to_path_buf(),
        });
    }

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
    fn write_file_if_missing_writes_then_keeps_existing_file() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let file_path = temp_dir.path().join("config.toml");

        write_file_if_missing(&file_path, "first\n").expect("file should be written");
        write_file_if_missing(&file_path, "second\n")
            .expect("existing file should remain untouched");

        let content = fs::read_to_string(&file_path).expect("file should be readable");
        assert_eq!(content, "first\n");
    }

    #[test]
    fn write_file_if_missing_errors_when_path_is_directory() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let dir_path = temp_dir.path().join("config");
        fs::create_dir_all(&dir_path).expect("directory should be created");

        let err =
            write_file_if_missing(&dir_path, "value\n").expect_err("directory path should fail");

        assert!(matches!(err, FsError::ExpectedFile { .. }));
    }
}
