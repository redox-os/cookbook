use serde::Serialize;
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::{self, Stdio},
    time::SystemTime,
};
use walkdir::{DirEntry, WalkDir};

//TODO: pub(crate) for all of these functions

pub fn remove_all(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
    .map_err(|err| format!("failed to remove '{}': {}\n{:?}", path.display(), err, err))
}

pub fn create_dir(dir: &Path) -> Result<(), String> {
    fs::create_dir(dir)
        .map_err(|err| format!("failed to create '{}': {}\n{:?}", dir.display(), err, err))
}

pub fn create_dir_clean(dir: &Path) -> Result<(), String> {
    if dir.is_dir() {
        remove_all(dir)?;
    }
    create_dir(dir)
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn symlink(original: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<(), String> {
    std::os::unix::fs::symlink(&original, &link).map_err(|err| {
        format!(
            "failed to symlink '{}' to '{}': {}\n{:?}",
            original.as_ref().display(),
            link.as_ref().display(),
            err,
            err
        )
    })
}

pub fn modified(path: &Path) -> Result<SystemTime, String> {
    let metadata = fs::metadata(path).map_err(|err| {
        format!(
            "failed to get metadata of '{}': {}\n{:#?}",
            path.display(),
            err,
            err
        )
    })?;
    metadata.modified().map_err(|err| {
        format!(
            "failed to get modified time of '{}': {}\n{:#?}",
            path.display(),
            err,
            err
        )
    })
}

pub fn modified_dir_inner<F: FnMut(&DirEntry) -> bool>(
    dir: &Path,
    filter: F,
) -> io::Result<SystemTime> {
    let mut newest = fs::metadata(dir)?.modified()?;
    for entry_res in WalkDir::new(dir).into_iter().filter_entry(filter) {
        let entry = entry_res?;
        let modified = entry.metadata()?.modified()?;
        if modified > newest {
            newest = modified;
        }
    }
    Ok(newest)
}

pub fn modified_dir(dir: &Path) -> Result<SystemTime, String> {
    modified_dir_inner(dir, |_| true).map_err(|err| {
        format!(
            "failed to get modified time of '{}': {}\n{:#?}",
            dir.display(),
            err,
            err
        )
    })
}

pub fn modified_dir_ignore_git(dir: &Path) -> Result<SystemTime, String> {
    modified_dir_inner(dir, |entry| {
        entry
            .file_name()
            .to_str()
            .map(|s| s != ".git")
            .unwrap_or(true)
    })
    .map_err(|err| {
        format!(
            "failed to get modified time of '{}': {}\n{:#?}",
            dir.display(),
            err,
            err
        )
    })
}

pub fn rename(src: &Path, dst: &Path) -> Result<(), String> {
    fs::rename(src, dst).map_err(|err| {
        format!(
            "failed to rename '{}' to '{}': {}\n{:?}",
            src.display(),
            dst.display(),
            err,
            err
        )
    })
}

pub fn run_command(mut command: process::Command) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|err| format!("failed to run {:?}: {}\n{:#?}", command, err, err))?;

    if !status.success() {
        return Err(format!(
            "failed to run {:?}: exited with status {}",
            command, status
        ));
    }

    Ok(())
}

pub fn run_command_stdin(mut command: process::Command, stdin_data: &[u8]) -> Result<(), String> {
    command.stdin(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|err| format!("failed to spawn {:?}: {}\n{:#?}", command, err, err))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(stdin_data).map_err(|err| {
            format!(
                "failed to write stdin of {:?}: {}\n{:#?}",
                command, err, err
            )
        })?;
    } else {
        return Err(format!("failed to find stdin of {:?}", command));
    }

    let status = child
        .wait()
        .map_err(|err| format!("failed to run {:?}: {}\n{:#?}", command, err, err))?;

    if !status.success() {
        return Err(format!(
            "failed to run {:?}: exited with status {}",
            command, status
        ));
    }

    Ok(())
}

pub fn serialize_and_write<T: Serialize>(file_path: &Path, content: &T) -> Result<(), String> {
    let toml_content = toml::to_string(content).map_err(|err| {
        format!(
            "Failed to serialize content for '{}': {}",
            file_path.display(),
            err
        )
    })?;

    fs::write(file_path, toml_content)
        .map_err(|err| format!("Failed to write to file '{}': {}", file_path.display(), err))?;
    Ok(())
}
