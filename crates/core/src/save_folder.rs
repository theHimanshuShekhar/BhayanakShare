//! The save folder: where an accepted Transfer is saved unless the Receiver picks another folder
//! for that Offer. It is a setting, kept across restarts; until set, it is the folder the Device
//! was configured with.

use std::{
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

/// The setting the save folder is stored under, as an absolute path.
pub(crate) const SETTING: &str = "save_folder";

/// Why a folder cannot be the save folder. Kinds, not sentences: the UI words them, and none
/// names a path (the user chose it, and an error may be logged).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SaveFolderProblem {
    /// A relative path (or none): it would mean whatever folder the app happens to run in.
    #[error("the path is not absolute")]
    NotAbsolute,
    /// There is something at that path that is not a folder.
    #[error("not a folder")]
    NotAFolder,
    /// It does not exist, and could not be made.
    #[error("the folder could not be created")]
    CannotCreate,
    /// BhayanakShare cannot put files in it.
    #[error("the folder is not writable")]
    NotWritable,
    /// The setting is kept as text, which that path is not.
    #[error("the path is not text")]
    NotText,
    /// It is missing because the drive it is on is not connected (see [`drive_missing`]).
    #[error("the folder is on a drive that is not connected")]
    NotConnected,
}

/// Where Linux mounts removable and extra drives: under one of these, a folder whose drive is not
/// mounted is missing, and the folder where it would be mounted is on the base's own filesystem.
#[cfg(unix)]
const MOUNT_BASES: [&str; 3] = ["/media", "/run/media", "/mnt"];
#[cfg(not(unix))]
const MOUNT_BASES: [&str; 0] = [];

/// Whether `folder`, which is not there, is missing because the drive it is on is not connected.
/// On Windows: its drive letter or network share is not there. Elsewhere: it is under a folder
/// drives are mounted in (`bases`), and nothing is mounted between them: the nearest part of it
/// that is there is on the base's own filesystem. A folder that was deleted from a drive that is
/// there is only missing.
pub(crate) fn drive_missing(folder: &Path, bases: &[&Path]) -> bool {
    if folder.exists() {
        return false;
    }
    #[cfg(windows)]
    {
        let _ = bases;
        use std::path::Component;
        let root: PathBuf =
            folder.components().take_while(|c| matches!(c, Component::Prefix(_) | Component::RootDir)).collect();
        !root.as_os_str().is_empty() && !root.exists()
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        let Some(base) = bases.iter().find(|base| folder.starts_with(base)) else { return false };
        let Some(nearest) = folder.ancestors().find(|a| a.exists()) else { return false };
        match (std::fs::metadata(base), std::fs::metadata(nearest)) {
            (Ok(base), Ok(nearest)) => base.dev() == nearest.dev(),
            _ => false,
        }
    }
}

/// [`drive_missing`] with this system's own mount bases.
pub(crate) fn on_missing_drive(folder: &Path) -> bool {
    drive_missing(folder, &MOUNT_BASES.map(Path::new))
}

/// Makes `folder` if it is missing and checks that a file can be written in it. Returns it as
/// kept in the setting. It must be an absolute path.
pub(crate) fn prepare(folder: &Path) -> Result<PathBuf, SaveFolderProblem> {
    prepare_under(folder, &MOUNT_BASES.map(Path::new))
}

/// [`prepare`] with the given mount bases (see [`drive_missing`]).
fn prepare_under(folder: &Path, bases: &[&Path]) -> Result<PathBuf, SaveFolderProblem> {
    if !folder.is_absolute() {
        return Err(SaveFolderProblem::NotAbsolute);
    }
    if folder.to_str().is_none() {
        return Err(SaveFolderProblem::NotText);
    }
    if folder.exists() && !folder.is_dir() {
        return Err(SaveFolderProblem::NotAFolder);
    }
    // A folder that can be made is only missing, wherever it is. One that cannot, where a drive
    // would be mounted, is on a drive that is not connected.
    std::fs::create_dir_all(folder).map_err(|_| {
        if drive_missing(folder, bases) { SaveFolderProblem::NotConnected } else { SaveFolderProblem::CannotCreate }
    })?;
    probe(folder).map_err(|_| SaveFolderProblem::NotWritable)?;
    Ok(folder.to_owned())
}

/// Writes a file in `folder` and removes it again. A folder's permission bits do not say whether
/// writing works (a read-only mount, a quota, an ACL), so this tries.
fn probe(folder: &Path) -> io::Result<()> {
    let name = format!(".bhayanakshare-write-check-{}", std::process::id());
    let path = folder.join(name);
    loop {
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => return std::fs::remove_file(&path),
            // A leftover of an earlier check that was cut short.
            Err(e) if e.kind() == ErrorKind::AlreadyExists => std::fs::remove_file(&path)?,
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_folder_is_made_and_the_check_leaves_nothing_in_it() {
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("a").join("b");
        assert_eq!(prepare(&folder), Ok(folder.clone()));
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 0);
        // A folder that is there is fine, and so is a leftover of an earlier check.
        std::fs::write(folder.join(format!(".bhayanakshare-write-check-{}", std::process::id())), b"").unwrap();
        assert_eq!(prepare(&folder), Ok(folder.clone()));
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_under_a_mount_base_with_nothing_mounted_is_on_a_drive_that_is_not_connected() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("media");
        let bases = [base.as_path()];
        std::fs::create_dir_all(base.join("me")).unwrap();
        // Nothing at the mount point, or an empty folder where the drive would be mounted.
        assert!(drive_missing(&base.join("me/USB/Received"), &bases));
        std::fs::create_dir_all(base.join("me/USB")).unwrap();
        assert!(drive_missing(&base.join("me/USB/Received"), &bases));
        // Anywhere else, a missing folder is only missing.
        assert!(!drive_missing(&tmp.path().join("home/me/Received"), &bases));
        // A folder that is there is on a drive that is there.
        assert!(!drive_missing(&base.join("me/USB"), &bases));
    }

    #[cfg(unix)]
    #[test]
    fn a_missing_folder_under_a_mount_base_is_made_when_it_can_be_and_otherwise_its_drive_is_not_connected() {
        use std::os::unix::fs::PermissionsExt as _;
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("mnt");
        let bases = [base.as_path()];
        // A folder of the user's own on the base's filesystem: it is only missing, so it is made.
        std::fs::create_dir_all(base.join("shared")).unwrap();
        let folder = base.join("shared/Received");
        assert_eq!(prepare_under(&folder, &bases), Ok(folder.clone()));
        assert!(folder.is_dir());
        // Where the drive would be mounted, which (as for any mount point) the user cannot write to.
        std::fs::create_dir_all(base.join("usb")).unwrap();
        std::fs::set_permissions(base.join("usb"), std::fs::Permissions::from_mode(0o555)).unwrap();
        let made = std::fs::create_dir(base.join("usb/probe")).is_ok(); // root writes anywhere
        let result = prepare_under(&base.join("usb/Received"), &bases);
        std::fs::set_permissions(base.join("usb"), std::fs::Permissions::from_mode(0o755)).unwrap();
        if !made {
            assert_eq!(result, Err(SaveFolderProblem::NotConnected));
        }
    }

    #[cfg(windows)]
    #[test]
    fn a_folder_on_a_drive_letter_that_is_not_there_is_on_a_drive_that_is_not_connected() {
        let free = (b'D'..=b'Z').rev().map(|l| format!("{}:\\", l as char)).find(|d| !Path::new(d).exists()).unwrap();
        assert!(drive_missing(&Path::new(&free).join("Received"), &[]));
        let tmp = tempfile::tempdir().unwrap();
        assert!(!drive_missing(&tmp.path().join("missing"), &[]));
    }

    #[test]
    fn a_path_that_is_not_absolute_is_refused_and_nothing_is_made() {
        assert_eq!(prepare(Path::new("received")), Err(SaveFolderProblem::NotAbsolute));
        assert_eq!(prepare(Path::new("")), Err(SaveFolderProblem::NotAbsolute));
        assert!(!Path::new("received").exists());
    }
}
