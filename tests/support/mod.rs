use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

pub struct TestDir {
    root: PathBuf,
}

#[allow(dead_code)]
impl TestDir {
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("rs-find-test-{}-{timestamp}", std::process::id()));
        fs::create_dir_all(&root).expect("failed to create test root");
        Self { root }
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn create_file(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent directories");
        }
        fs::write(&path, contents).expect("failed to write file");
        path
    }

    pub fn create_dir(&self, relative: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(&path).expect("failed to create directory");
        path
    }

    #[cfg(unix)]
    pub fn create_dir_symlink(&self, target_relative: &str, link_relative: &str) -> PathBuf {
        let link = self.root.join(link_relative);
        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent).expect("failed to create symlink parent directory");
        }
        symlink(self.root.join(target_relative), &link).expect("failed to create symlink");
        link
    }

    #[cfg(unix)]
    pub fn make_unreadable_dir(&self, relative: &str) -> PathBuf {
        let path = self.create_dir(relative);
        let mut permissions = fs::metadata(&path)
            .expect("failed to stat directory")
            .permissions();
        permissions.set_mode(0o000);
        fs::set_permissions(&path, permissions).expect("failed to remove directory permissions");
        path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        #[cfg(unix)]
        restore_permissions(&self.root);
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn restore_permissions(path: &Path) {
    if let Ok(metadata) = fs::symlink_metadata(path)
        && metadata.is_dir()
        && !metadata.file_type().is_symlink()
    {
        if let Ok(read_dir) = fs::read_dir(path) {
            for child in read_dir.flatten() {
                restore_permissions(&child.path());
            }
        }
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        let _ = fs::set_permissions(path, permissions);
    }
}
