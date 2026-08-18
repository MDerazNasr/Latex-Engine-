//! Bounded session storage for Kitty local file transmission.

use std::collections::HashMap;
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::Write as _;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use latex_render_svg::RasterLimits;
use sha2::{Digest as _, Sha256};

use crate::ImageSource;

const DIRECTORY_ATTEMPTS: usize = 32;
const DIRECTORY_PREFIX: &str = "latex-engine-tty-graphics-protocol";
const DEFAULT_MAX_FILES: usize = 256;
const DEFAULT_MAX_TOTAL_BYTES: usize = 128 * 1024 * 1024;
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

/// Resource limits for one session owned local PNG store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalPngStoreLimits {
    max_files: NonZeroUsize,
    max_total_bytes: NonZeroUsize,
}

impl LocalPngStoreLimits {
    /// Creates nonzero file and byte limits.
    pub fn new(max_files: usize, max_total_bytes: usize) -> Result<Self, LocalStoreError> {
        Ok(Self {
            max_files: NonZeroUsize::new(max_files).ok_or(LocalStoreError::InvalidLimits)?,
            max_total_bytes: NonZeroUsize::new(max_total_bytes)
                .ok_or(LocalStoreError::InvalidLimits)?,
        })
    }

    /// Returns the maximum number of distinct PNG files.
    pub const fn max_files(self) -> NonZeroUsize {
        self.max_files
    }

    /// Returns the maximum total PNG bytes retained by the session.
    pub const fn max_total_bytes(self) -> NonZeroUsize {
        self.max_total_bytes
    }
}

impl Default for LocalPngStoreLimits {
    fn default() -> Self {
        Self {
            max_files: NonZeroUsize::new(DEFAULT_MAX_FILES).expect("default files are nonzero"),
            max_total_bytes: NonZeroUsize::new(DEFAULT_MAX_TOTAL_BYTES)
                .expect("default bytes are nonzero"),
        }
    }
}

/// Failure while creating or using session local image storage.
#[derive(Debug)]
pub enum LocalStoreError {
    /// One or more resource limits were zero.
    InvalidLimits,
    /// PNG bytes were empty, malformed, or globally oversized.
    InvalidPng,
    /// The configured file count or total byte capacity was reached.
    CapacityExceeded,
    /// A private session directory could not be created.
    CreateDirectory(std::io::Error),
    /// A content addressed file unexpectedly already existed.
    FileCollision,
    /// A local PNG could not be created or completely written.
    WriteFile(std::io::Error),
}

impl Display for LocalStoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLimits => formatter.write_str("local PNG store limits must be nonzero"),
            Self::InvalidPng => formatter.write_str("local image is not a bounded PNG"),
            Self::CapacityExceeded => formatter.write_str("local PNG store capacity was exceeded"),
            Self::CreateDirectory(error) => {
                write!(
                    formatter,
                    "private local PNG directory could not be created: {error}"
                )
            }
            Self::FileCollision => formatter.write_str("local PNG file name collided"),
            Self::WriteFile(error) => write!(formatter, "local PNG could not be written: {error}"),
        }
    }
}

impl std::error::Error for LocalStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreateDirectory(error) | Self::WriteFile(error) => Some(error),
            Self::InvalidLimits
            | Self::InvalidPng
            | Self::CapacityExceeded
            | Self::FileCollision => None,
        }
    }
}

/// Owns content addressed PNG files for one terminal presentation session.
#[derive(Debug)]
pub struct LocalPngStore {
    directory: PathBuf,
    entries: HashMap<[u8; 32], PathBuf>,
    used_bytes: usize,
    limits: LocalPngStoreLimits,
}

impl LocalPngStore {
    /// Creates a private store beneath the operating system temporary directory.
    pub fn create(limits: LocalPngStoreLimits) -> Result<Self, LocalStoreError> {
        Self::create_in(env::temp_dir(), limits)
    }

    /// Creates a private store beneath an explicit temporary parent.
    pub fn create_in(
        parent: impl AsRef<Path>,
        limits: LocalPngStoreLimits,
    ) -> Result<Self, LocalStoreError> {
        let parent = fs::canonicalize(parent).map_err(LocalStoreError::CreateDirectory)?;
        for _ in 0..DIRECTORY_ATTEMPTS {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let directory = parent.join(format!(
                "{DIRECTORY_PREFIX}-{}-{sequence}",
                std::process::id()
            ));
            match create_private_directory(&directory) {
                Ok(()) => {
                    return Ok(Self {
                        directory,
                        entries: HashMap::new(),
                        used_bytes: 0,
                        limits,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(LocalStoreError::CreateDirectory(error)),
            }
        }
        Err(LocalStoreError::CreateDirectory(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "private directory names were exhausted",
        )))
    }

    /// Stores one PNG once and returns a source retained for this session.
    pub fn store_png(&mut self, png: &[u8]) -> Result<ImageSource, LocalStoreError> {
        validate_png(png)?;
        let digest: [u8; 32] = Sha256::digest(png).into();
        if let Some(path) = self.entries.get(&digest) {
            return Ok(ImageSource::LocalPng(path.clone()));
        }
        let next_bytes = self
            .used_bytes
            .checked_add(png.len())
            .ok_or(LocalStoreError::CapacityExceeded)?;
        if self.entries.len() >= self.limits.max_files.get()
            || next_bytes > self.limits.max_total_bytes.get()
        {
            return Err(LocalStoreError::CapacityExceeded);
        }

        let path = self.directory.join(format!("{}.png", hex_digest(&digest)));
        let mut file = create_private_file(&path)?;
        if let Err(error) = file.write_all(png).and_then(|()| file.flush()) {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(LocalStoreError::WriteFile(error));
        }
        drop(file);
        self.entries.insert(digest, path.clone());
        self.used_bytes = next_bytes;
        Ok(ImageSource::LocalPng(path))
    }

    /// Returns the number of distinct files retained by this session.
    pub fn file_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the total PNG payload bytes retained by this session.
    pub fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    /// Returns the uniquely owned directory for diagnostics.
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

impl Drop for LocalPngStore {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn validate_png(png: &[u8]) -> Result<(), LocalStoreError> {
    latex_render_svg::validate_png(png, RasterLimits::default())
        .map(|_| ())
        .map_err(|_| LocalStoreError::InvalidPng)
}

fn hex_digest(digest: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(output, "{byte:02x}").expect("writing to a string cannot fail");
    }
    output
}

fn create_private_directory(path: &Path) -> Result<(), std::io::Error> {
    let mut builder = DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700);
    }
    builder.create(path)
}

fn create_private_file(path: &Path) -> Result<File, LocalStoreError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    options.open(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            LocalStoreError::FileCollision
        } else {
            LocalStoreError::WriteFile(error)
        }
    })
}
