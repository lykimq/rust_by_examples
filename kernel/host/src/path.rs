//! Enforcing correct encoding of [storage paths].
//!
//! A storage path can be written to, or read from by the kernel - and *may* correspond
//! to a sequence of bytes in the [runtime] storage.
//!
//! [runtime]: crate::runtime
//! [storage paths]: Path

/// The separator byte between *steps* in a path.
pub const PATH_SEPARATOR: u8 = b'/';

/// The maximum size (in bytes) of a path.
pub const PATH_MAX_SIZE: usize = 250 - DURABLE_STORAGE_PREFIX_INNER.len();

/// The implicit prefix of all durable storage, as slice.
const DURABLE_STORAGE_PREFIX_INNER: &[u8] = b"/durable";

/// The implicit prefix of all durable storage.
pub const DURABLE_STORAGE_PREFIX: RefPath =
    RefPath::assert_from(DURABLE_STORAGE_PREFIX_INNER);

/// The contents of `/kernel/next` must be a path pointing to the kernel to be loaded
/// on the next *reboot*.
///
/// By default, this will point to `/kernel/boot.wasm`.
pub const PATH_KERNEL_NEXT: RefPath = RefPath::assert_from(b"/kernel/next");

/// The *wasm-encoded* binary blob of either the currently running kernel, or the next
/// kernel to be rebooted into.
pub const PATH_KERNEL_BOOT: RefPath = RefPath::assert_from(b"/kernel/boot.wasm");

/// Marker trait for methods on types representing *path-encodings*.
///
/// Path encoding maintains the following invariants:
/// - paths begin with `b'/'`.
/// - paths are a sequence of non-empty *steps*, separated by a single `b'/'`.
/// - steps are a sequence of either ascii-encoded alphanumeric bytes, or `b'.'`.
/// - the maximum length of a path is [PATH_MAX_SIZE].
/// where `b'/'` is the [PATH_SEPARATOR].
///
/// i.e. path encoding may be summarised by the regex `(\/[A-Za-z0-9.]+)+` up to a maximum
/// [PATH_MAX_SIZE] bytes.
///
/// # Safety
/// [`Path`] is unsafe to implement, as other code (e.g. [`Runtime`]) rely on any
/// `T: impl Path` being correctly path-encoded.
///
/// [`Runtime`]: crate::runtime::Runtime
pub unsafe trait Path {
    /// Returns a read-only reference to the underlying path-encoded byte-slice.
    fn as_bytes(&self) -> &[u8];

    /// Returns a pointer to the beginning of the path.
    fn as_ptr(&self) -> *const u8 {
        self.as_bytes().as_ptr()
    }

    /// Returns the size of the path *in bytes*.
    fn size(&self) -> usize {
        let size = self.as_bytes().len();
        debug_assert!(size <= PATH_MAX_SIZE);
        size
    }

    /// Returns the length of the path, as decomposed into a sequence of *steps*.
    fn len_steps(&self) -> usize {
        self.as_bytes()
            .iter()
            .filter(|b| b == &&PATH_SEPARATOR)
            .count()
    }
}

/// Possible path validation errors.
#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub enum PathError {
    /// Path contains no steps.
    PathEmpty,
    /// Path must be at most [PATH_MAX_SIZE] bytes in size.
    ///
    /// [PATH_MAX_SIZE]: self::PATH_MAX_SIZE
    PathTooLong,
    /// Path must begin with [PATH_SEPARATOR].
    ///
    /// [PATH_SEPARATOR]: self::PATH_SEPARATOR
    InvalidStart,
    /// Path must not contain empty steps.
    InvalidEmptyStep,
    /// Steps must be a sequence of bytes such that every byte is in `[A-Za-z0-9.]`
    InvalidByteInStep,
}

/// Representation of a [`Path`] which borrows its underlying byte-sequence.
///
/// Useful when either:
/// - a path is known and may be statically declared at compile-time.
/// - only a *view* over a path is required: `RefPath` & `&RefPath` may be freely passed
///   around and copied, since they don't own their underlying data.
///
/// Otherwise, you can use [OwnedPath].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RefPath<'a> {
    inner: &'a str,
}

impl<'a> RefPath<'a> {
    /// Constructs a [`RefPath`] from a byte slice.
    ///
    /// # Panics
    /// `panics` if the byte slice does not represent a valid path-encoding.
    /// See [`Path`].  For example, the following would
    ///
    /// # Examples
    ///
    /// It is possible to define a well-encoded path at compile time:
    /// ```
    /// # use host::path::RefPath;
    /// const PATH: RefPath<'static> = RefPath::assert_from("/valid/path".as_bytes());
    /// ```
    ///
    /// But the following would fail to compile:
    /// ```compile_fail
    /// # use host::path::RefPath;
    /// const PATH: RefPath<'static> = RefPath::assert_from("invalid//path".as_bytes());
    /// ```
    ///
    /// And this would panic at runtime:
    /// ```should_panic
    /// # use host::path::RefPath;
    /// let path = RefPath::assert_from("!&(*(".as_bytes());
    /// ```
    pub const fn assert_from(path: &[u8]) -> RefPath {
        assert_is_valid_path(path);

        RefPath {
            inner: unsafe { core::str::from_utf8_unchecked(path) },
        }
    }
}

unsafe impl Path for RefPath<'_> {
    fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

const fn is_allowed_step_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'.'
}

const fn assert_is_valid_path(path: &[u8]) {
    match validate_path(path) {
        Err(PathError::PathEmpty) => panic!("Path must contain at least one empty step"),
        Err(PathError::PathTooLong) => panic!("Path contained too many bytes"),
        Err(PathError::InvalidStart) => panic!("Path must begin with a path separator"),
        Err(PathError::InvalidEmptyStep) => panic!("Path steps must be non empty"),
        Err(PathError::InvalidByteInStep) => {
            panic!("Path step bytes must be ascii_alphanumeric or \"= b'.'\"")
        }
        Ok(()) => (),
    }
}

const fn validate_path(path: &[u8]) -> Result<(), PathError> {
    match path {
        [] => Err(PathError::PathEmpty),
        [PATH_SEPARATOR] | [.., PATH_SEPARATOR] => Err(PathError::InvalidEmptyStep),
        _ if path.len() > PATH_MAX_SIZE => Err(PathError::PathTooLong),
        [PATH_SEPARATOR, ..] => {
            let mut i = 1;
            let size = path.len();

            while i < size {
                match (path[i - 1], path[i]) {
                    (PATH_SEPARATOR, PATH_SEPARATOR) => {
                        return Err(PathError::InvalidEmptyStep)
                    }
                    (_, PATH_SEPARATOR) => (),
                    (_, c) if !is_allowed_step_byte(c) => {
                        return Err(PathError::InvalidByteInStep)
                    }
                    _ => (),
                }
                i += 1;
            }
            Ok(())
        }
        _ => Err(PathError::InvalidStart),
    }
}

impl<'a> TryFrom<&'a [u8]> for RefPath<'a> {
    type Error = PathError;

    fn try_from(slice: &'a [u8]) -> Result<RefPath, PathError> {
        let _: () = validate_path(slice)?;

        // SAFETY: we've validated that every byte is either alphanumeric or SEPARATOR
        let inner = unsafe { core::str::from_utf8_unchecked(slice) };

        Ok(RefPath { inner })
    }
}

#[cfg(feature = "alloc")]
pub use owned::*;
#[cfg(feature = "alloc")]
mod owned {
    use super::{validate_path, Path, PathError, RefPath};
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    /// Representation of a [`Path`] which *owns* its underlying path-encoded byte sequence.
    ///
    /// Useful when a new path is being constructed at runtime, which is not a sub-path of an
    /// already existing path (in which case you may use [RefPath]).
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct OwnedPath {
        inner: String,
    }

    impl OwnedPath {
        /// Constructs an [`OwnedPath`] from an arbitrary sequence of bytes.
        ///
        /// # Safety
        /// Validation that `bytes` are a validly encoded path is bypassed.  The bytes
        /// **must** be correctly path-encoded.  See [`Path`].
        pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Self {
            Self {
                inner: String::from_utf8_unchecked(bytes),
            }
        }
    }

    unsafe impl Path for OwnedPath {
        fn as_bytes(&self) -> &[u8] {
            self.inner.as_bytes()
        }
    }

    impl<'a> From<&RefPath<'a>> for OwnedPath {
        fn from(path: &RefPath<'a>) -> Self {
            Self {
                inner: path.inner.to_string(),
            }
        }
    }

    impl<'a> From<RefPath<'a>> for OwnedPath {
        fn from(path: RefPath<'a>) -> Self {
            Self {
                inner: path.inner.to_string(),
            }
        }
    }

    impl<'a> From<&'a OwnedPath> for RefPath<'a> {
        fn from(path: &'a OwnedPath) -> Self {
            Self { inner: &path.inner }
        }
    }

    impl TryFrom<Vec<u8>> for OwnedPath {
        type Error = PathError;

        fn try_from(bytes: Vec<u8>) -> Result<OwnedPath, PathError> {
            let _: () = validate_path(&bytes)?;

            // SAFETY: we've validated that every byte is either alphanumeric or SEPARATOR
            let inner = unsafe { String::from_utf8_unchecked(bytes) };

            Ok(OwnedPath { inner })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_slice_path_cannot_be_empty() {
        let empty: Vec<u8> = vec![];
        let result = OwnedPath::try_from(empty);

        assert_eq!(Err(PathError::PathEmpty), result);
    }

    #[test]
    fn from_slice_path_err_on_sep_not_first_byte() {
        for v in u8::MIN..=u8::MAX {
            match v {
                PATH_SEPARATOR => continue,
                v => {
                    let path = vec![v, b'r', b'e', b's', b't'];
                    let result = OwnedPath::try_from(path);

                    assert_eq!(Err(PathError::InvalidStart), result);
                }
            }
        }
    }

    #[test]
    fn from_slice_path_err_on_sep_last_byte() {
        let path: Vec<u8> = vec![PATH_SEPARATOR, b'p', b'a', b't', b'h', PATH_SEPARATOR];
        let result = OwnedPath::try_from(path);

        assert_eq!(Err(PathError::InvalidEmptyStep), result);
    }

    #[test]
    fn from_slice_err_on_duplicate_separator() {
        let path = "/this/path/is/completely/fine/except/for/the//in/the/middle/of/it";
        let result: Result<RefPath, PathError> = path.as_bytes().try_into();

        assert_eq!(Err(PathError::InvalidEmptyStep), result);
    }

    #[test]
    fn from_slice_too_long() {
        let bytes = [b'i'; PATH_MAX_SIZE - 1];

        let mut path = vec![PATH_SEPARATOR];
        path.extend_from_slice(&bytes);

        let result: Result<RefPath, _> = path.as_slice().try_into();
        assert!(result.is_ok());

        path.extend_from_slice(&bytes[0..1]);

        let result: Result<RefPath, _> = path.as_slice().try_into();
        assert_eq!(Err(PathError::PathTooLong), result);
    }

    #[test]
    fn from_slice_ok() {
        let path = "/Th1s/PATH/1s/absolut3Ly/f1ne/and/sh0u.ld/.../work";
        let result = RefPath::try_from(path.as_bytes());

        let expected = RefPath { inner: path };

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn store_path_get_len_steps() {
        let path = "/this/path/is/absolutely/fine/and/should.must/work";
        let result = RefPath::try_from(path.as_bytes()).unwrap();

        assert_eq!(8, result.len_steps());
    }
}
