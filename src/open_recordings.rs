use anyhow::{Context, Result};
use std::{fs::create_dir_all, path::Path, process::Command};

pub fn open_recordings(directory: &Path) -> Result<()> {
    prepare_recordings_directory(directory)?;
    let (program, args) = opener_command(directory);
    Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("failed to launch the file manager for {}", directory.display()))?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("file manager failed to open {}", directory.display()))
}

pub fn prepare_recordings_directory(directory: &Path) -> Result<()> {
    create_dir_all(directory).with_context(|| {
        format!("failed to create recordings directory {}", directory.display())
    })?;
    Ok(())
}

fn opener_command(directory: &Path) -> (&'static str, [&Path; 1]) {
    #[cfg(target_os = "macos")]
    {
        ("open", [directory])
    }
    #[cfg(target_os = "windows")]
    {
        ("explorer", [directory])
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        ("xdg-open", [directory])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn creates_the_recordings_directory() {
        let root = tempdir().unwrap();
        let directory = root.path().join("nested").join("recordings");
        prepare_recordings_directory(&directory).unwrap();
        assert!(directory.is_dir());
    }

    #[test]
    fn builds_a_single_argument_for_the_platform_opener() {
        let directory = PathBuf::from("recordings with spaces");
        let (program, args) = opener_command(&directory);
        assert!(!program.is_empty());
        assert_eq!(args, [&directory]);
    }
}
