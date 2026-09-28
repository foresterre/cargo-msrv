//! The sources of the collection: bundled with this crate, read from a local file, or
//! downloaded (with the `download` feature).

use crate::archive::ALIGNMENT;
use crate::{ParseError, StdSinceIndex};
use rkyv::util::AlignedVec;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Aligns the bundled archive, so it can be read in place.
#[repr(C, align(16))]
struct Aligned<T: ?Sized>(T);

const _: () = assert!(align_of::<Aligned<[u8; 0]>>() == ALIGNMENT);

static BUNDLED: &Aligned<[u8]> = &Aligned(*include_bytes!("../data/std_since.rkyv"));

/// The location of the latest collection, which is updated by the maintainers of cargo-msrv.
#[cfg(feature = "download")]
pub const LATEST_URL: &str = "https://raw.githubusercontent.com/foresterre/cargo-msrv/main/crates/cargo-msrv-std-since/data/std_since.rkyv";

/// How long a downloaded collection is used, before a newer one is downloaded.
#[cfg(feature = "download")]
pub const CACHE_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

/// A downloaded collection is limited in size, so a wrong response can't exhaust memory.
#[cfg(feature = "download")]
const DOWNLOAD_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("unable to read the collection '{path}': {source}")]
    Read { path: PathBuf, source: io::Error },

    #[error("unable to parse the collection from '{origin}': {source}")]
    Parse { origin: String, source: ParseError },

    #[cfg(feature = "download")]
    #[error("unable to download the collection from '{url}': {source}")]
    Download {
        url: String,
        // Boxed, since the error is large compared to the other variants.
        source: Box<ureq::Error>,
    },

    #[error("unable to write the collection to the cache '{path}': {source}")]
    WriteCache { path: PathBuf, source: io::Error },
}

enum Bytes {
    Bundled(&'static [u8]),
    Owned(AlignedVec<ALIGNMENT>),
}

/// The aligned bytes of a collection, and where they came from.
///
/// The bytes are validated by [`Archive::index`], which returns an index that reads them in
/// place.
pub struct Archive {
    bytes: Bytes,
    origin: String,
}

impl Archive {
    fn owned(bytes: &[u8], origin: String) -> Self {
        let mut aligned = AlignedVec::<ALIGNMENT>::with_capacity(bytes.len());
        aligned.extend_from_slice(bytes);

        Self {
            bytes: Bytes::Owned(aligned),
            origin,
        }
    }

    fn bytes(&self) -> &[u8] {
        match &self.bytes {
            Bytes::Bundled(bytes) => bytes,
            Bytes::Owned(bytes) => bytes,
        }
    }

    /// Where the collection came from, e.g. a path or a URL.
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// Validates the archive, and returns an index which reads it in place.
    pub fn index(&self) -> Result<StdSinceIndex<'_>, SourceError> {
        StdSinceIndex::from_bytes(self.bytes()).map_err(|source| SourceError::Parse {
            origin: self.origin.clone(),
            source,
        })
    }
}

/// The collection bundled with this crate.
pub fn bundled() -> Archive {
    Archive {
        bytes: Bytes::Bundled(&BUNDLED.0),
        origin: "the bundled collection".to_string(),
    }
}

/// Reads a collection from a local file, e.g. one written by the generator.
pub fn from_file(path: &Path) -> Result<Archive, SourceError> {
    let bytes = fs::read(path).map_err(|source| SourceError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    Ok(Archive::owned(&bytes, path.display().to_string()))
}

/// Downloads the latest collection, unless the collection in the `cache_file` was
/// downloaded less than [`CACHE_MAX_AGE`] ago.
///
/// A downloaded collection is validated, and then written to the `cache_file`.
#[cfg(feature = "download")]
pub fn latest(cache_file: &Path) -> Result<Archive, SourceError> {
    if let Some(archive) = fresh_cache(cache_file) {
        return Ok(archive);
    }

    let bytes = download(LATEST_URL)?;
    let archive = Archive::owned(&bytes, LATEST_URL.to_string());
    archive.index()?;

    let write_cache_error = |source| SourceError::WriteCache {
        path: cache_file.to_path_buf(),
        source,
    };

    if let Some(dir) = cache_file.parent() {
        fs::create_dir_all(dir).map_err(write_cache_error)?;
    }
    fs::write(cache_file, bytes).map_err(write_cache_error)?;

    Ok(archive)
}

/// Returns the cached collection, if it's recent and valid. Otherwise, a new one is
/// downloaded, so an unreadable or invalid cache is not an error.
#[cfg(feature = "download")]
fn fresh_cache(cache_file: &Path) -> Option<Archive> {
    let modified = fs::metadata(cache_file).ok()?.modified().ok()?;
    let age = modified.elapsed().ok()?;

    if age > CACHE_MAX_AGE {
        return None;
    }

    let archive = from_file(cache_file).ok()?;
    archive.index().ok()?;

    Some(archive)
}

#[cfg(feature = "download")]
fn download(url: &str) -> Result<Vec<u8>, SourceError> {
    let download_error = |source| SourceError::Download {
        url: url.to_string(),
        source: Box::new(source),
    };

    ureq::get(url)
        .call()
        .map_err(download_error)?
        .body_mut()
        .with_config()
        .limit(DOWNLOAD_LIMIT)
        .read_to_vec()
        .map_err(download_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CollectionBuilder, Item, ItemKind};
    use assert_fs::prelude::*;

    fn collection() -> Vec<u8> {
        let mut builder = CollectionBuilder::default();
        let item = Item {
            kind: ItemKind::Function,
            since: semver::Version::new(1, 66, 0),
        };
        builder.insert_item("core::hint::black_box", item);

        builder.to_bytes().unwrap().to_vec()
    }

    #[test]
    fn bundled_collection_is_valid() {
        let archive = bundled();
        let index = archive.index().unwrap();

        assert!(index.toolchain().is_some());
        assert_eq!(
            index.resolve_path(&["std", "option", "Option", "is_some_and"]),
            Some(semver::Version::new(1, 70, 0))
        );
    }

    #[test]
    fn reads_from_file() {
        let file = assert_fs::NamedTempFile::new("std_since.rkyv").unwrap();
        file.write_binary(&collection()).unwrap();

        let archive = from_file(file.path()).unwrap();

        assert!(archive.index().unwrap().toolchain().is_none());
    }

    #[test]
    fn missing_file() {
        let dir = assert_fs::TempDir::new().unwrap();

        assert!(matches!(
            from_file(&dir.path().join("missing.rkyv")),
            Err(SourceError::Read { .. })
        ));
    }

    #[test]
    fn invalid_file() {
        let file = assert_fs::NamedTempFile::new("std_since.rkyv").unwrap();
        file.write_str("not a collection").unwrap();

        let archive = from_file(file.path()).unwrap();

        assert!(matches!(archive.index(), Err(SourceError::Parse { .. })));
    }

    #[cfg(feature = "download")]
    #[test]
    fn latest_uses_a_fresh_cache() {
        let cache = assert_fs::NamedTempFile::new("std_since.rkyv").unwrap();
        cache.write_binary(&collection()).unwrap();

        let archive = latest(cache.path()).unwrap();

        assert_eq!(archive.origin(), cache.path().display().to_string());
    }
}
