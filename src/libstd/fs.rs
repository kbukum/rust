// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Filesystem manipulation operations.
//!
//! This module contains basic methods to manipulate the contents of the local
//! filesystem. All methods in this module represent cross-platform filesystem
//! operations. Extra platform-specific functionality can be found in the
//! extension traits of `std::os::$platform`.

#![stable(feature = "rust1", since = "1.0.0")]

use fmt;
use ffi::OsString;
use io::{self, SeekFrom, Seek, Read, Write};
use path::{Path, PathBuf};
use sys::fs as fs_imp;
use sys_common::{AsInnerMut, FromInner, AsInner, IntoInner};
use time::SystemTime;

/// A reference to an open file on the filesystem.
///
/// An instance of a `File` can be read and/or written depending on what options
/// it was opened with. Files also implement `Seek` to alter the logical cursor
/// that the file contains internally.
///
/// Files are automatically closed when they go out of scope.
///
/// # Examples
///
/// ```no_run
/// use std::io::prelude::*;
/// use std::fs::File;
///
/// # fn foo() -> std::io::Result<()> {
/// let mut f = try!(File::create("foo.txt"));
/// try!(f.write_all(b"Hello, world!"));
///
/// let mut f = try!(File::open("foo.txt"));
/// let mut s = String::new();
/// try!(f.read_to_string(&mut s));
/// assert_eq!(s, "Hello, world!");
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct File {
    inner: fs_imp::File,
}

/// Metadata information about a file.
///
/// This structure is returned from the [`metadata`] function or method and
/// represents known metadata about a file such as its permissions, size,
/// modification times, etc.
///
/// [`metadata`]: fn.metadata.html
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Clone)]
pub struct Metadata(fs_imp::FileAttr);

/// Iterator over the entries in a directory.
///
/// This iterator is returned from the [`read_dir`] function of this module and
/// will yield instances of `io::Result<DirEntry>`. Through a [`DirEntry`]
/// information like the entry's path and possibly other metadata can be
/// learned.
///
/// [`read_dir`]: fn.read_dir.html
/// [`DirEntry`]: struct.DirEntry.html
///
/// # Errors
///
/// This [`io::Result`] will be an `Err` if there's some sort of intermittent
/// IO error during iteration.
///
/// [`io::Result`]: ../io/type.Result.html
#[stable(feature = "rust1", since = "1.0.0")]
pub struct ReadDir(fs_imp::ReadDir);

/// Entries returned by the [`ReadDir`] iterator.
///
/// [`ReadDir`]: struct.ReadDir.html
///
/// An instance of `DirEntry` represents an entry inside of a directory on the
/// filesystem. Each entry can be inspected via methods to learn about the full
/// path or possibly other metadata through per-platform extension traits.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct DirEntry(fs_imp::DirEntry);

/// Options and flags which can be used to configure how a file is opened.
///
/// This builder exposes the ability to configure how a [`File`] is opened and
/// what operations are permitted on the open file. The [`File::open`] and
/// [`File::create`] methods are aliases for commonly used options using this
/// builder.
///
/// [`File`]: struct.File.html
/// [`File::open`]: struct.File.html#method.open
/// [`File::create`]: struct.File.html#method.create
///
/// Generally speaking, when using `OpenOptions`, you'll first call [`new()`],
/// then chain calls to methods to set each option, then call [`open()`],
/// passing the path of the file you're trying to open. This will give you a
/// [`io::Result`][result] with a [`File`][file] inside that you can further
/// operate on.
///
/// [`new()`]: struct.OpenOptions.html#method.new
/// [`open()`]: struct.OpenOptions.html#method.open
/// [result]: ../io/type.Result.html
/// [file]: struct.File.html
///
/// # Examples
///
/// Opening a file to read:
///
/// ```no_run
/// use std::fs::OpenOptions;
///
/// let file = OpenOptions::new().read(true).open("foo.txt");
/// ```
///
/// Opening a file for both reading and writing, as well as creating it if it
/// doesn't exist:
///
/// ```no_run
/// use std::fs::OpenOptions;
///
/// let file = OpenOptions::new()
///             .read(true)
///             .write(true)
///             .create(true)
///             .open("foo.txt");
/// ```
#[derive(Clone)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct OpenOptions(fs_imp::OpenOptions);

/// Representation of the various permissions on a file.
///
/// This module only currently provides one bit of information, [`readonly`],
/// which is exposed on all currently supported platforms. Unix-specific
/// functionality, such as mode bits, is available through the
/// `os::unix::PermissionsExt` trait.
///
/// [`readonly`]: struct.Permissions.html#method.readonly
#[derive(Clone, PartialEq, Eq, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Permissions(fs_imp::FilePermissions);

/// A structure representing a type of file with accessors for each file type.
/// It is returned by [`Metadata::file_type`] method.
///
/// [`Metadata::file_type`]: struct.Metadata.html#method.file_type
#[stable(feature = "file_type", since = "1.1.0")]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileType(fs_imp::FileType);

/// A builder used to create directories in various manners.
///
/// This builder also supports platform-specific options.
#[stable(feature = "dir_builder", since = "1.6.0")]
pub struct DirBuilder {
    inner: fs_imp::DirBuilder,
    recursive: bool,
}

impl File {
    /// Attempts to open a file in read-only mode.
    ///
    /// See the [`OpenOptions::open`] method for more details.
    ///
    /// # Errors
    ///
    /// This function will return an error if `path` does not already exist.
    /// Other errors may also be returned according to [`OpenOptions::open`].
    ///
    /// [`OpenOptions::open`]: struct.OpenOptions.html#method.open
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = try!(File::open("foo.txt"));
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
        OpenOptions::new().read(true).open(path.as_ref())
    }

    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist,
    /// and will truncate it if it does.
    ///
    /// See the [`OpenOptions::open`] function for more details.
    ///
    /// [`OpenOptions::open`]: struct.OpenOptions.html#method.open
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = try!(File::create("foo.txt"));
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<File> {
        OpenOptions::new().write(true).create(true).truncate(true).open(path.as_ref())
    }

    /// Attempts to sync all OS-internal metadata to disk.
    ///
    /// This function will attempt to ensure that all in-core data reaches the
    /// filesystem before returning.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::prelude::*;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = try!(File::create("foo.txt"));
    /// try!(f.write_all(b"Hello, world!"));
    ///
    /// try!(f.sync_all());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn sync_all(&self) -> io::Result<()> {
        self.inner.fsync()
    }

    /// This function is similar to [`sync_all`], except that it may not
    /// synchronize file metadata to the filesystem.
    ///
    /// This is intended for use cases that must synchronize content, but don't
    /// need the metadata on disk. The goal of this method is to reduce disk
    /// operations.
    ///
    /// Note that some platforms may simply implement this in terms of
    /// [`sync_all`].
    ///
    /// [`sync_all`]: struct.File.html#method.sync_all
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::prelude::*;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = try!(File::create("foo.txt"));
    /// try!(f.write_all(b"Hello, world!"));
    ///
    /// try!(f.sync_data());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn sync_data(&self) -> io::Result<()> {
        self.inner.datasync()
    }

    /// Truncates or extends the underlying file, updating the size of
    /// this file to become `size`.
    ///
    /// If the `size` is less than the current file's size, then the file will
    /// be shrunk. If it is greater than the current file's size, then the file
    /// will be extended to `size` and have all of the intermediate data filled
    /// in with 0s.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file is not opened for writing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = try!(File::create("foo.txt"));
    /// try!(f.set_len(10));
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.inner.truncate(size)
    }

    /// Queries metadata about the underlying file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = try!(File::open("foo.txt"));
    /// let metadata = try!(f.metadata());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.inner.file_attr().map(Metadata)
    }

    /// Creates a new independently owned handle to the underlying file.
    ///
    /// The returned `File` is a reference to the same state that this object
    /// references. Both handles will read and write with the same cursor
    /// position.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = try!(File::open("foo.txt"));
    /// let file_copy = try!(f.try_clone());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "file_try_clone", since = "1.9.0")]
    pub fn try_clone(&self) -> io::Result<File> {
        Ok(File {
            inner: self.inner.duplicate()?
        })
    }
}

impl AsInner<fs_imp::File> for File {
    fn as_inner(&self) -> &fs_imp::File { &self.inner }
}
impl FromInner<fs_imp::File> for File {
    fn from_inner(f: fs_imp::File) -> File {
        File { inner: f }
    }
}
impl IntoInner<fs_imp::File> for File {
    fn into_inner(self) -> fs_imp::File {
        self.inner
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Read for &'a File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Write for &'a File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Seek for &'a File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let mut options = OpenOptions::new();
    /// let file = options.read(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new() -> OpenOptions {
        OpenOptions(fs_imp::OpenOptions::new())
    }

    /// Sets the option for read access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `read`-able if opened.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().read(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.0.read(read); self
    }

    /// Sets the option for write access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `write`-able if opened.
    ///
    /// If the file already exists, any write calls on it will overwrite its
    /// contents, without truncating it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.0.write(write); self
    }

    /// Sets the option for the append mode.
    ///
    /// This option, when true, means that writes will append to a file instead
    /// of overwriting previous contents.
    /// Note that setting `.write(true).append(true)` has the same effect as
    /// setting only `.append(true)`.
    ///
    /// For most filesystems, the operating system guarantees that all writes are
    /// atomic: no writes get mangled because another process writes at the same
    /// time.
    ///
    /// One maybe obvious note when using append-mode: make sure that all data
    /// that belongs together is written to the file in one operation. This
    /// can be done by concatenating strings before passing them to `write()`,
    /// or using a buffered writer (with a buffer of adequate size),
    /// and calling `flush()` when the message is complete.
    ///
    /// If a file is opened with both read and append access, beware that after
    /// opening, and after every write, the position for reading may be set at the
    /// end of the file. So, before writing, save the current position (using
    /// `seek(SeekFrom::Current(0))`, and restore it before the next read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().append(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.0.append(append); self
    }

    /// Sets the option for truncating a previous file.
    ///
    /// If a file is successfully opened with this option set it will truncate
    /// the file to 0 length if it already exists.
    ///
    /// The file must be opened with write access for truncate to work.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).truncate(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        self.0.truncate(truncate); self
    }

    /// Sets the option for creating a new file.
    ///
    /// This option indicates whether a new file will be created if the file
    /// does not yet already exist.
    ///
    /// In order for the file to be created, `write` or `append` access must
    /// be used.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).create(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.0.create(create); self
    }

    /// Sets the option to always create a new file.
    ///
    /// This option indicates whether a new file will be created.
    /// No file is allowed to exist at the target location, also no (dangling)
    /// symlink.
    ///
    /// This option is useful because it is atomic. Otherwise between checking
    /// whether a file exists and creating a new one, the file may have been
    /// created by another process (a TOCTOU race condition / attack).
    ///
    /// If `.create_new(true)` is set, `.create()` and `.truncate()` are
    /// ignored.
    ///
    /// The file must be opened with write or append access in order to create
    /// a new file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true)
    ///                              .create_new(true)
    ///                              .open("foo.txt");
    /// ```
    #[stable(feature = "expand_open_options2", since = "1.9.0")]
    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        self.0.create_new(create_new); self
    }

    /// Opens a file at `path` with the options specified by `self`.
    ///
    /// # Errors
    ///
    /// This function will return an error under a number of different
    /// circumstances, to include but not limited to:
    ///
    /// * Opening a file that does not exist without setting `create` or
    ///   `create_new`.
    /// * Attempting to open a file with access that the user lacks
    ///   permissions for
    /// * Filesystem-level errors (full disk, etc)
    /// * Invalid combinations of open options (truncate without write access,
    ///   no access mode set, etc)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        self._open(path.as_ref())
    }

    fn _open(&self, path: &Path) -> io::Result<File> {
        let inner = fs_imp::File::open(path, &self.0)?;
        Ok(File { inner: inner })
    }
}

impl AsInnerMut<fs_imp::OpenOptions> for OpenOptions {
    fn as_inner_mut(&mut self) -> &mut fs_imp::OpenOptions { &mut self.0 }
}

impl Metadata {
    /// Returns the file type for this metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    ///
    /// println!("{:?}", metadata.file_type());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "file_type", since = "1.1.0")]
    pub fn file_type(&self) -> FileType {
        FileType(self.0.file_type())
    }

    /// Returns whether this metadata is for a directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    ///
    /// assert!(!metadata.is_dir());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn is_dir(&self) -> bool { self.file_type().is_dir() }

    /// Returns whether this metadata is for a regular file.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    ///
    /// assert!(metadata.is_file());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn is_file(&self) -> bool { self.file_type().is_file() }

    /// Returns the size of the file, in bytes, this metadata is for.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    ///
    /// assert_eq!(0, metadata.len());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn len(&self) -> u64 { self.0.size() }

    /// Returns the permissions of the file this metadata is for.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    ///
    /// assert!(!metadata.permissions().readonly());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn permissions(&self) -> Permissions {
        Permissions(self.0.perm())
    }

    /// Returns the last modification time listed in this metadata.
    ///
    /// The returned value corresponds to the `mtime` field of `stat` on Unix
    /// platforms and the `ftLastWriteTime` field on Windows platforms.
    ///
    /// # Errors
    ///
    /// This field may not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    ///
    /// if let Ok(time) = metadata.modified() {
    ///     println!("{:?}", time);
    /// } else {
    ///     println!("Not supported on this platform");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "fs_time", since = "1.10.0")]
    pub fn modified(&self) -> io::Result<SystemTime> {
        self.0.modified().map(FromInner::from_inner)
    }

    /// Returns the last access time of this metadata.
    ///
    /// The returned value corresponds to the `atime` field of `stat` on Unix
    /// platforms and the `ftLastAccessTime` field on Windows platforms.
    ///
    /// Note that not all platforms will keep this field update in a file's
    /// metadata, for example Windows has an option to disable updating this
    /// time when files are accessed and Linux similarly has `noatime`.
    ///
    /// # Errors
    ///
    /// This field may not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    ///
    /// if let Ok(time) = metadata.accessed() {
    ///     println!("{:?}", time);
    /// } else {
    ///     println!("Not supported on this platform");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "fs_time", since = "1.10.0")]
    pub fn accessed(&self) -> io::Result<SystemTime> {
        self.0.accessed().map(FromInner::from_inner)
    }

    /// Returns the creation time listed in the this metadata.
    ///
    /// The returned value corresponds to the `birthtime` field of `stat` on
    /// Unix platforms and the `ftCreationTime` field on Windows platforms.
    ///
    /// # Errors
    ///
    /// This field may not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    ///
    /// if let Ok(time) = metadata.created() {
    ///     println!("{:?}", time);
    /// } else {
    ///     println!("Not supported on this platform");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "fs_time", since = "1.10.0")]
    pub fn created(&self) -> io::Result<SystemTime> {
        self.0.created().map(FromInner::from_inner)
    }
}

impl AsInner<fs_imp::FileAttr> for Metadata {
    fn as_inner(&self) -> &fs_imp::FileAttr { &self.0 }
}

impl Permissions {
    /// Returns whether these permissions describe a readonly file.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = try!(File::create("foo.txt"));
    /// let metadata = try!(f.metadata());
    ///
    /// assert_eq!(false, metadata.permissions().readonly());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn readonly(&self) -> bool { self.0.readonly() }

    /// Modifies the readonly flag for this set of permissions.
    ///
    /// This operation does **not** modify the filesystem. To modify the
    /// filesystem use the `fs::set_permissions` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let f = try!(File::create("foo.txt"));
    /// let metadata = try!(f.metadata());
    /// let mut permissions = metadata.permissions();
    ///
    /// permissions.set_readonly(true);
    ///
    /// // filesystem doesn't change
    /// assert_eq!(false, metadata.permissions().readonly());
    ///
    /// // just this particular `permissions`.
    /// assert_eq!(true, permissions.readonly());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn set_readonly(&mut self, readonly: bool) {
        self.0.set_readonly(readonly)
    }
}

impl FileType {
    /// Test whether this file type represents a directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    /// let file_type = metadata.file_type();
    ///
    /// assert_eq!(file_type.is_dir(), false);
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "file_type", since = "1.1.0")]
    pub fn is_dir(&self) -> bool { self.0.is_dir() }

    /// Test whether this file type represents a regular file.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    /// let file_type = metadata.file_type();
    ///
    /// assert_eq!(file_type.is_file(), true);
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "file_type", since = "1.1.0")]
    pub fn is_file(&self) -> bool { self.0.is_file() }

    /// Test whether this file type represents a symbolic link.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = try!(fs::metadata("foo.txt"));
    /// let file_type = metadata.file_type();
    ///
    /// assert_eq!(file_type.is_symlink(), false);
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "file_type", since = "1.1.0")]
    pub fn is_symlink(&self) -> bool { self.0.is_symlink() }
}

impl AsInner<fs_imp::FileType> for FileType {
    fn as_inner(&self) -> &fs_imp::FileType { &self.0 }
}

impl FromInner<fs_imp::FilePermissions> for Permissions {
    fn from_inner(f: fs_imp::FilePermissions) -> Permissions {
        Permissions(f)
    }
}

impl AsInner<fs_imp::FilePermissions> for Permissions {
    fn as_inner(&self) -> &fs_imp::FilePermissions { &self.0 }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        self.0.next().map(|entry| entry.map(DirEntry))
    }
}

impl DirEntry {
    /// Returns the full path to the file that this entry represents.
    ///
    /// The full path is created by joining the original path to `read_dir`
    /// with the filename of this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// # fn foo() -> std::io::Result<()> {
    /// for entry in try!(fs::read_dir(".")) {
    ///     let dir = try!(entry);
    ///     println!("{:?}", dir.path());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// This prints output like:
    ///
    /// ```text
    /// "./whatever.txt"
    /// "./foo.html"
    /// "./hello_world.rs"
    /// ```
    ///
    /// The exact text, of course, depends on what files you have in `.`.
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn path(&self) -> PathBuf { self.0.path() }

    /// Return the metadata for the file that this entry points at.
    ///
    /// This function will not traverse symlinks if this entry points at a
    /// symlink.
    ///
    /// # Platform-specific behavior
    ///
    /// On Windows this function is cheap to call (no extra system calls
    /// needed), but on Unix platforms this function is the equivalent of
    /// calling `symlink_metadata` on the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    ///
    /// if let Ok(entries) = fs::read_dir(".") {
    ///     for entry in entries {
    ///         if let Ok(entry) = entry {
    ///             // Here, `entry` is a `DirEntry`.
    ///             if let Ok(metadata) = entry.metadata() {
    ///                 // Now let's show our entry's permissions!
    ///                 println!("{:?}: {:?}", entry.path(), metadata.permissions());
    ///             } else {
    ///                 println!("Couldn't get metadata for {:?}", entry.path());
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[stable(feature = "dir_entry_ext", since = "1.1.0")]
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.0.metadata().map(Metadata)
    }

    /// Return the file type for the file that this entry points at.
    ///
    /// This function will not traverse symlinks if this entry points at a
    /// symlink.
    ///
    /// # Platform-specific behavior
    ///
    /// On Windows and most Unix platforms this function is free (no extra
    /// system calls needed), but some Unix platforms may require the equivalent
    /// call to `symlink_metadata` to learn about the target file type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    ///
    /// if let Ok(entries) = fs::read_dir(".") {
    ///     for entry in entries {
    ///         if let Ok(entry) = entry {
    ///             // Here, `entry` is a `DirEntry`.
    ///             if let Ok(file_type) = entry.file_type() {
    ///                 // Now let's show our entry's file type!
    ///                 println!("{:?}: {:?}", entry.path(), file_type);
    ///             } else {
    ///                 println!("Couldn't get file type for {:?}", entry.path());
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[stable(feature = "dir_entry_ext", since = "1.1.0")]
    pub fn file_type(&self) -> io::Result<FileType> {
        self.0.file_type().map(FileType)
    }

    /// Returns the bare file name of this directory entry without any other
    /// leading path component.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    ///
    /// if let Ok(entries) = fs::read_dir(".") {
    ///     for entry in entries {
    ///         if let Ok(entry) = entry {
    ///             // Here, `entry` is a `DirEntry`.
    ///             println!("{:?}", entry.file_name());
    ///         }
    ///     }
    /// }
    /// ```
    #[stable(feature = "dir_entry_ext", since = "1.1.0")]
    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }
}

#[stable(feature = "dir_entry_debug", since = "1.13.0")]
impl fmt::Debug for DirEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("DirEntry")
            .field(&self.path())
            .finish()
    }
}

impl AsInner<fs_imp::DirEntry> for DirEntry {
    fn as_inner(&self) -> &fs_imp::DirEntry { &self.0 }
}

/// Removes a file from the filesystem.
///
/// Note that there is no
/// guarantee that the file is immediately deleted (e.g. depending on
/// platform, other open file descriptors may prevent immediate removal).
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `unlink` function on Unix
/// and the `DeleteFile` function on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` points to a directory.
/// * The user lacks permissions to remove the file.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// try!(fs::remove_file("a.txt"));
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs_imp::unlink(path.as_ref())
}

/// Given a path, query the file system to get information about a file,
/// directory, etc.
///
/// This function will traverse symbolic links to query information about the
/// destination file.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `stat` function on Unix
/// and the `GetFileAttributesEx` function on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The user lacks permissions to perform `metadata` call on `path`.
/// * `path` does not exist.
///
/// # Examples
///
/// ```rust
/// # fn foo() -> std::io::Result<()> {
/// use std::fs;
///
/// let attr = try!(fs::metadata("/some/file/path.txt"));
/// // inspect attr ...
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
    fs_imp::stat(path.as_ref()).map(Metadata)
}

/// Query the metadata about a file without following symlinks.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `lstat` function on Unix
/// and the `GetFileAttributesEx` function on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The user lacks permissions to perform `metadata` call on `path`.
/// * `path` does not exist.
///
/// # Examples
///
/// ```rust
/// # fn foo() -> std::io::Result<()> {
/// use std::fs;
///
/// let attr = try!(fs::symlink_metadata("/some/file/path.txt"));
/// // inspect attr ...
/// # Ok(())
/// # }
/// ```
#[stable(feature = "symlink_metadata", since = "1.1.0")]
pub fn symlink_metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
    fs_imp::lstat(path.as_ref()).map(Metadata)
}

/// Rename a file or directory to a new name, replacing the original file if
/// `to` already exists.
///
/// This will not work if the new name is on a different mount point.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `rename` function on Unix
/// and the `MoveFileEx` function with the `MOVEFILE_REPLACE_EXISTING` flag on Windows.
///
/// Because of this, the behavior when both `from` and `to` exist differs. On
/// Unix, if `from` is a directory, `to` must also be an (empty) directory. If
/// `from` is not a directory, `to` must also be not a directory. In contrast,
/// on Windows, `from` can be anything, but `to` must *not* be a directory.
///
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `from` does not exist.
/// * The user lacks permissions to view contents.
/// * `from` and `to` are on separate filesystems.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// try!(fs::rename("a.txt", "b.txt")); // Rename a.txt to b.txt
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
    fs_imp::rename(from.as_ref(), to.as_ref())
}

/// Copies the contents of one file to another. This function will also
/// copy the permission bits of the original file to the destination file.
///
/// This function will **overwrite** the contents of `to`.
///
/// Note that if `from` and `to` both point to the same file, then the file
/// will likely get truncated by this operation.
///
/// On success, the total number of bytes copied is returned.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `open` function in Unix
/// with `O_RDONLY` for `from` and `O_WRONLY`, `O_CREAT`, and `O_TRUNC` for `to`.
/// `O_CLOEXEC` is set for returned file descriptors.
/// On Windows, this function currently corresponds to `CopyFileEx`.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The `from` path is not a file.
/// * The `from` file does not exist.
/// * The current process does not have the permission rights to access
///   `from` or write `to`.
///
/// # Examples
///
/// ```no_run
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// try!(fs::copy("foo.txt", "bar.txt"));  // Copy foo.txt to bar.txt
/// # Ok(()) }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    fs_imp::copy(from.as_ref(), to.as_ref())
}

/// Creates a new hard link on the filesystem.
///
/// The `dst` path will be a link pointing to the `src` path. Note that systems
/// often require these two paths to both be located on the same filesystem.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `link` function on Unix
/// and the `CreateHardLink` function on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The `src` path is not a file or doesn't exist.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// try!(fs::hard_link("a.txt", "b.txt")); // Hard link a.txt to b.txt
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    fs_imp::link(src.as_ref(), dst.as_ref())
}

/// Creates a new symbolic link on the filesystem.
///
/// The `dst` path will be a symbolic link pointing to the `src` path.
/// On Windows, this will be a file symlink, not a directory symlink;
/// for this reason, the platform-specific `std::os::unix::fs::symlink`
/// and `std::os::windows::fs::{symlink_file, symlink_dir}` should be
/// used instead to make the intent explicit.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// try!(fs::soft_link("a.txt", "b.txt"));
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_deprecated(since = "1.1.0",
             reason = "replaced with std::os::unix::fs::symlink and \
                       std::os::windows::fs::{symlink_file, symlink_dir}")]
pub fn soft_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    fs_imp::symlink(src.as_ref(), dst.as_ref())
}

/// Reads a symbolic link, returning the file that the link points to.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `readlink` function on Unix
/// and the `CreateFile` function with `FILE_FLAG_OPEN_REPARSE_POINT` and
/// `FILE_FLAG_BACKUP_SEMANTICS` flags on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` is not a symbolic link.
/// * `path` does not exist.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// let path = try!(fs::read_link("a.txt"));
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn read_link<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    fs_imp::readlink(path.as_ref())
}

/// Returns the canonical form of a path with all intermediate components
/// normalized and symbolic links resolved.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `realpath` function on Unix
/// and the `CreateFile` and `GetFinalPathNameByHandle` functions on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` does not exist.
/// * A component in path is not a directory.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// let path = try!(fs::canonicalize("../a/../foo.txt"));
/// # Ok(())
/// # }
/// ```
#[stable(feature = "fs_canonicalize", since = "1.5.0")]
pub fn canonicalize<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    fs_imp::canonicalize(path.as_ref())
}

/// Creates a new, empty directory at the provided path
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `mkdir` function on Unix
/// and the `CreateDirectory` function on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * User lacks permissions to create directory at `path`.
/// * `path` already exists.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// try!(fs::create_dir("/some/dir"));
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn create_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    DirBuilder::new().create(path.as_ref())
}

/// Recursively create a directory and all of its parent components if they
/// are missing.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `mkdir` function on Unix
/// and the `CreateDirectory` function on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * If any directory in the path specified by `path`
/// does not already exist and it could not be created otherwise. The specific
/// error conditions for when a directory is being created (after it is
/// determined to not exist) are outlined by `fs::create_dir`.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// try!(fs::create_dir_all("/some/dir"));
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    DirBuilder::new().recursive(true).create(path.as_ref())
}

/// Removes an existing, empty directory.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `rmdir` function on Unix
/// and the `RemoveDirectory` function on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The user lacks permissions to remove the directory at the provided `path`.
/// * The directory isn't empty.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// try!(fs::remove_dir("/some/dir"));
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn remove_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs_imp::rmdir(path.as_ref())
}

/// Removes a directory at this path, after removing all its contents. Use
/// carefully!
///
/// This function does **not** follow symbolic links and it will simply remove the
/// symbolic link itself.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to `opendir`, `lstat`, `rm` and `rmdir` functions on Unix
/// and the `FindFirstFile`, `GetFileAttributesEx`, `DeleteFile`, and `RemoveDirectory` functions
/// on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// See `file::remove_file` and `fs::remove_dir`.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// try!(fs::remove_dir_all("/some/dir"));
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs_imp::remove_dir_all(path.as_ref())
}

/// Returns an iterator over the entries within a directory.
///
/// The iterator will yield instances of [`io::Result`]`<`[`DirEntry`]`>`.
/// New errors may be encountered after an iterator is initially constructed.
///
/// [`io::Result`]: ../io/type.Result.html
/// [`DirEntry`]: struct.DirEntry.html
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `opendir` function on Unix
/// and the `FindFirstFile` function on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The provided `path` doesn't exist.
/// * The process lacks permissions to view the contents.
/// * The `path` points at a non-directory file.
///
/// # Examples
///
/// ```
/// use std::io;
/// use std::fs::{self, DirEntry};
/// use std::path::Path;
///
/// // one possible implementation of walking a directory only visiting files
/// fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry)) -> io::Result<()> {
///     if dir.is_dir() {
///         for entry in try!(fs::read_dir(dir)) {
///             let entry = try!(entry);
///             let path = entry.path();
///             if path.is_dir() {
///                 try!(visit_dirs(&path, cb));
///             } else {
///                 cb(&entry);
///             }
///         }
///     }
///     Ok(())
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<ReadDir> {
    fs_imp::readdir(path.as_ref()).map(ReadDir)
}

/// Changes the permissions found on a file or a directory.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `chmod` function on Unix
/// and the `SetFileAttributes` function on Windows.
/// Note that, this [may change in the future][changes].
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` does not exist.
/// * The user lacks the permission to change attributes of the file.
///
/// # Examples
///
/// ```
/// # fn foo() -> std::io::Result<()> {
/// use std::fs;
///
/// let mut perms = try!(fs::metadata("foo.txt")).permissions();
/// perms.set_readonly(true);
/// try!(fs::set_permissions("foo.txt", perms));
/// # Ok(())
/// # }
/// ```
#[stable(feature = "set_permissions", since = "1.1.0")]
pub fn set_permissions<P: AsRef<Path>>(path: P, perm: Permissions)
                                       -> io::Result<()> {
    fs_imp::set_perm(path.as_ref(), perm.0)
}

impl DirBuilder {
    /// Creates a new set of options with default mode/security settings for all
    /// platforms and also non-recursive.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::DirBuilder;
    ///
    /// let builder = DirBuilder::new();
    /// ```
    #[stable(feature = "dir_builder", since = "1.6.0")]
    pub fn new() -> DirBuilder {
        DirBuilder {
            inner: fs_imp::DirBuilder::new(),
            recursive: false,
        }
    }

    /// Indicate that directories create should be created recursively, creating
    /// all parent directories if they do not exist with the same security and
    /// permissions settings.
    ///
    /// This option defaults to `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::DirBuilder;
    ///
    /// let mut builder = DirBuilder::new();
    /// builder.recursive(true);
    /// ```
    #[stable(feature = "dir_builder", since = "1.6.0")]
    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    /// Create the specified directory with the options configured in this
    /// builder.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::{self, DirBuilder};
    ///
    /// let path = "/tmp/foo/bar/baz";
    /// DirBuilder::new()
    ///     .recursive(true)
    ///     .create(path).unwrap();
    ///
    /// assert!(fs::metadata(path).unwrap().is_dir());
    /// ```
    #[stable(feature = "dir_builder", since = "1.6.0")]
    pub fn create<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self._create(path.as_ref())
    }

    fn _create(&self, path: &Path) -> io::Result<()> {
        if self.recursive {
            self.create_dir_all(path)
        } else {
            self.inner.mkdir(path)
        }
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        if path == Path::new("") || path.is_dir() { return Ok(()) }
        if let Some(p) = path.parent() {
            self.create_dir_all(p)?
        }
        self.inner.mkdir(path)
    }
}

impl AsInnerMut<fs_imp::DirBuilder> for DirBuilder {
    fn as_inner_mut(&mut self) -> &mut fs_imp::DirBuilder {
        &mut self.inner
    }
}

#[cfg(all(test, not(target_os = "emscripten")))]
mod tests {
    use io::prelude::*;

    use fs::{self, File, OpenOptions};
    use io::{ErrorKind, SeekFrom};
    use path::Path;
    use rand::{StdRng, Rng};
    use str;
    use sys_common::io::test::{TempDir, tmpdir};

    #[cfg(windows)]
    use os::windows::fs::{symlink_dir, symlink_file};
    #[cfg(windows)]
    use sys::fs::symlink_junction;
    #[cfg(unix)]
    use os::unix::fs::symlink as symlink_dir;
    #[cfg(unix)]
    use os::unix::fs::symlink as symlink_file;
    #[cfg(unix)]
    use os::unix::fs::symlink as symlink_junction;

    macro_rules! check { ($e:expr) => (
        match $e {
            Ok(t) => t,
            Err(e) => panic!("{} failed with: {}", stringify!($e), e),
        }
    ) }

    macro_rules! error { ($e:expr, $s:expr) => (
        match $e {
            Ok(_) => panic!("Unexpected success. Should've been: {:?}", $s),
            Err(ref err) => assert!(err.to_string().contains($s),
                                    format!("`{}` did not contain `{}`", err, $s))
        }
    ) }

    // Several test fail on windows if the user does not have permission to
    // create symlinks (the `SeCreateSymbolicLinkPrivilege`). Instead of
    // disabling these test on Windows, use this function to test whether we
    // have permission, and return otherwise. This way, we still don't run these
    // tests most of the time, but at least we do if the user has the right
    // permissions.
    pub fn got_symlink_permission(tmpdir: &TempDir) -> bool {
        if cfg!(unix) { return true }
        let link = tmpdir.join("some_hopefully_unique_link_name");

        match symlink_file(r"nonexisting_target", link) {
            Ok(_) => true,
            Err(ref err) =>
                if err.to_string().contains("A required privilege is not held by the client.") {
                    false
                } else {
                    true
                }
        }
    }

    #[test]
    fn file_test_io_smoke_test() {
        let message = "it's alright. have a good time";
        let tmpdir = tmpdir();
        let filename = &tmpdir.join("file_rt_io_file_test.txt");
        {
            let mut write_stream = check!(File::create(filename));
            check!(write_stream.write(message.as_bytes()));
        }
        {
            let mut read_stream = check!(File::open(filename));
            let mut read_buf = [0; 1028];
            let read_str = match check!(read_stream.read(&mut read_buf)) {
                0 => panic!("shouldn't happen"),
                n => str::from_utf8(&read_buf[..n]).unwrap().to_string()
            };
            assert_eq!(read_str, message);
        }
        check!(fs::remove_file(filename));
    }

    #[test]
    fn invalid_path_raises() {
        let tmpdir = tmpdir();
        let filename = &tmpdir.join("file_that_does_not_exist.txt");
        let result = File::open(filename);

        if cfg!(unix) {
            error!(result, "No such file or directory");
        }
        if cfg!(windows) {
            error!(result, "The system cannot find the file specified");
        }
    }

    #[test]
    fn file_test_iounlinking_invalid_path_should_raise_condition() {
        let tmpdir = tmpdir();
        let filename = &tmpdir.join("file_another_file_that_does_not_exist.txt");

        let result = fs::remove_file(filename);

        if cfg!(unix) {
            error!(result, "No such file or directory");
        }
        if cfg!(windows) {
            error!(result, "The system cannot find the file specified");
        }
    }

    #[test]
    fn file_test_io_non_positional_read() {
        let message: &str = "ten-four";
        let mut read_mem = [0; 8];
        let tmpdir = tmpdir();
        let filename = &tmpdir.join("file_rt_io_file_test_positional.txt");
        {
            let mut rw_stream = check!(File::create(filename));
            check!(rw_stream.write(message.as_bytes()));
        }
        {
            let mut read_stream = check!(File::open(filename));
            {
                let read_buf = &mut read_mem[0..4];
                check!(read_stream.read(read_buf));
            }
            {
                let read_buf = &mut read_mem[4..8];
                check!(read_stream.read(read_buf));
            }
        }
        check!(fs::remove_file(filename));
        let read_str = str::from_utf8(&read_mem).unwrap();
        assert_eq!(read_str, message);
    }

    #[test]
    fn file_test_io_seek_and_tell_smoke_test() {
        let message = "ten-four";
        let mut read_mem = [0; 4];
        let set_cursor = 4 as u64;
        let tell_pos_pre_read;
        let tell_pos_post_read;
        let tmpdir = tmpdir();
        let filename = &tmpdir.join("file_rt_io_file_test_seeking.txt");
        {
            let mut rw_stream = check!(File::create(filename));
            check!(rw_stream.write(message.as_bytes()));
        }
        {
            let mut read_stream = check!(File::open(filename));
            check!(read_stream.seek(SeekFrom::Start(set_cursor)));
            tell_pos_pre_read = check!(read_stream.seek(SeekFrom::Current(0)));
            check!(read_stream.read(&mut read_mem));
            tell_pos_post_read = check!(read_stream.seek(SeekFrom::Current(0)));
        }
        check!(fs::remove_file(filename));
        let read_str = str::from_utf8(&read_mem).unwrap();
        assert_eq!(read_str, &message[4..8]);
        assert_eq!(tell_pos_pre_read, set_cursor);
        assert_eq!(tell_pos_post_read, message.len() as u64);
    }

    #[test]
    fn file_test_io_seek_and_write() {
        let initial_msg =   "food-is-yummy";
        let overwrite_msg =    "-the-bar!!";
        let final_msg =     "foo-the-bar!!";
        let seek_idx = 3;
        let mut read_mem = [0; 13];
        let tmpdir = tmpdir();
        let filename = &tmpdir.join("file_rt_io_file_test_seek_and_write.txt");
        {
            let mut rw_stream = check!(File::create(filename));
            check!(rw_stream.write(initial_msg.as_bytes()));
            check!(rw_stream.seek(SeekFrom::Start(seek_idx)));
            check!(rw_stream.write(overwrite_msg.as_bytes()));
        }
        {
            let mut read_stream = check!(File::open(filename));
            check!(read_stream.read(&mut read_mem));
        }
        check!(fs::remove_file(filename));
        let read_str = str::from_utf8(&read_mem).unwrap();
        assert!(read_str == final_msg);
    }

    #[test]
    fn file_test_io_seek_shakedown() {
        //                   01234567890123
        let initial_msg =   "qwer-asdf-zxcv";
        let chunk_one: &str = "qwer";
        let chunk_two: &str = "asdf";
        let chunk_three: &str = "zxcv";
        let mut read_mem = [0; 4];
        let tmpdir = tmpdir();
        let filename = &tmpdir.join("file_rt_io_file_test_seek_shakedown.txt");
        {
            let mut rw_stream = check!(File::create(filename));
            check!(rw_stream.write(initial_msg.as_bytes()));
        }
        {
            let mut read_stream = check!(File::open(filename));

            check!(read_stream.seek(SeekFrom::End(-4)));
            check!(read_stream.read(&mut read_mem));
            assert_eq!(str::from_utf8(&read_mem).unwrap(), chunk_three);

            check!(read_stream.seek(SeekFrom::Current(-9)));
            check!(read_stream.read(&mut read_mem));
            assert_eq!(str::from_utf8(&read_mem).unwrap(), chunk_two);

            check!(read_stream.seek(SeekFrom::Start(0)));
            check!(read_stream.read(&mut read_mem));
            assert_eq!(str::from_utf8(&read_mem).unwrap(), chunk_one);
        }
        check!(fs::remove_file(filename));
    }

    #[test]
    fn file_test_io_eof() {
        let tmpdir = tmpdir();
        let filename = tmpdir.join("file_rt_io_file_test_eof.txt");
        let mut buf = [0; 256];
        {
            let oo = OpenOptions::new().create_new(true).write(true).read(true).clone();
            let mut rw = check!(oo.open(&filename));
            assert_eq!(check!(rw.read(&mut buf)), 0);
            assert_eq!(check!(rw.read(&mut buf)), 0);
        }
        check!(fs::remove_file(&filename));
    }

    #[test]
    #[cfg(unix)]
    fn file_test_io_read_write_at() {
        use os::unix::fs::FileExt;

        let tmpdir = tmpdir();
        let filename = tmpdir.join("file_rt_io_file_test_read_write_at.txt");
        let mut buf = [0; 256];
        let write1 = "asdf";
        let write2 = "qwer-";
        let write3 = "-zxcv";
        let content = "qwer-asdf-zxcv";
        {
            let oo = OpenOptions::new().create_new(true).write(true).read(true).clone();
            let mut rw = check!(oo.open(&filename));
            assert_eq!(check!(rw.write_at(write1.as_bytes(), 5)), write1.len());
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 0);
            assert_eq!(check!(rw.read_at(&mut buf, 5)), write1.len());
            assert_eq!(str::from_utf8(&buf[..write1.len()]), Ok(write1));
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 0);
            assert_eq!(check!(rw.read_at(&mut buf[..write2.len()], 0)), write2.len());
            assert_eq!(str::from_utf8(&buf[..write2.len()]), Ok("\0\0\0\0\0"));
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 0);
            assert_eq!(check!(rw.write(write2.as_bytes())), write2.len());
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 5);
            assert_eq!(check!(rw.read(&mut buf)), write1.len());
            assert_eq!(str::from_utf8(&buf[..write1.len()]), Ok(write1));
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
            assert_eq!(check!(rw.read_at(&mut buf[..write2.len()], 0)), write2.len());
            assert_eq!(str::from_utf8(&buf[..write2.len()]), Ok(write2));
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
            assert_eq!(check!(rw.write_at(write3.as_bytes(), 9)), write3.len());
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
        }
        {
            let mut read = check!(File::open(&filename));
            assert_eq!(check!(read.read_at(&mut buf, 0)), content.len());
            assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
            assert_eq!(check!(read.seek(SeekFrom::Current(0))), 0);
            assert_eq!(check!(read.seek(SeekFrom::End(-5))), 9);
            assert_eq!(check!(read.read_at(&mut buf, 0)), content.len());
            assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
            assert_eq!(check!(read.seek(SeekFrom::Current(0))), 9);
            assert_eq!(check!(read.read(&mut buf)), write3.len());
            assert_eq!(str::from_utf8(&buf[..write3.len()]), Ok(write3));
            assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
            assert_eq!(check!(read.read_at(&mut buf, 0)), content.len());
            assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
            assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
            assert_eq!(check!(read.read_at(&mut buf, 14)), 0);
            assert_eq!(check!(read.read_at(&mut buf, 15)), 0);
            assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
        }
        check!(fs::remove_file(&filename));
    }

    #[test]
    #[cfg(windows)]
    fn file_test_io_seek_read_write() {
        use os::windows::fs::FileExt;

        let tmpdir = tmpdir();
        let filename = tmpdir.join("file_rt_io_file_test_seek_read_write.txt");
        let mut buf = [0; 256];
        let write1 = "asdf";
        let write2 = "qwer-";
        let write3 = "-zxcv";
        let content = "qwer-asdf-zxcv";
        {
            let oo = OpenOptions::new().create_new(true).write(true).read(true).clone();
            let mut rw = check!(oo.open(&filename));
            assert_eq!(check!(rw.seek_write(write1.as_bytes(), 5)), write1.len());
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
            assert_eq!(check!(rw.seek_read(&mut buf, 5)), write1.len());
            assert_eq!(str::from_utf8(&buf[..write1.len()]), Ok(write1));
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
            assert_eq!(check!(rw.seek(SeekFrom::Start(0))), 0);
            assert_eq!(check!(rw.write(write2.as_bytes())), write2.len());
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 5);
            assert_eq!(check!(rw.read(&mut buf)), write1.len());
            assert_eq!(str::from_utf8(&buf[..write1.len()]), Ok(write1));
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
            assert_eq!(check!(rw.seek_read(&mut buf[..write2.len()], 0)), write2.len());
            assert_eq!(str::from_utf8(&buf[..write2.len()]), Ok(write2));
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 5);
            assert_eq!(check!(rw.seek_write(write3.as_bytes(), 9)), write3.len());
            assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 14);
        }
        {
            let mut read = check!(File::open(&filename));
            assert_eq!(check!(read.seek_read(&mut buf, 0)), content.len());
            assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
            assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
            assert_eq!(check!(read.seek(SeekFrom::End(-5))), 9);
            assert_eq!(check!(read.seek_read(&mut buf, 0)), content.len());
            assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
            assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
            assert_eq!(check!(read.seek(SeekFrom::End(-5))), 9);
            assert_eq!(check!(read.read(&mut buf)), write3.len());
            assert_eq!(str::from_utf8(&buf[..write3.len()]), Ok(write3));
            assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
            assert_eq!(check!(read.seek_read(&mut buf, 0)), content.len());
            assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
            assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
            assert_eq!(check!(read.seek_read(&mut buf, 14)), 0);
            assert_eq!(check!(read.seek_read(&mut buf, 15)), 0);
        }
        check!(fs::remove_file(&filename));
    }

    #[test]
    fn file_test_stat_is_correct_on_is_file() {
        let tmpdir = tmpdir();
        let filename = &tmpdir.join("file_stat_correct_on_is_file.txt");
        {
            let mut opts = OpenOptions::new();
            let mut fs = check!(opts.read(true).write(true)
                                    .create(true).open(filename));
            let msg = "hw";
            fs.write(msg.as_bytes()).unwrap();

            let fstat_res = check!(fs.metadata());
            assert!(fstat_res.is_file());
        }
        let stat_res_fn = check!(fs::metadata(filename));
        assert!(stat_res_fn.is_file());
        let stat_res_meth = check!(filename.metadata());
        assert!(stat_res_meth.is_file());
        check!(fs::remove_file(filename));
    }

    #[test]
    fn file_test_stat_is_correct_on_is_dir() {
        let tmpdir = tmpdir();
        let filename = &tmpdir.join("file_stat_correct_on_is_dir");
        check!(fs::create_dir(filename));
        let stat_res_fn = check!(fs::metadata(filename));
        assert!(stat_res_fn.is_dir());
        let stat_res_meth = check!(filename.metadata());
        assert!(stat_res_meth.is_dir());
        check!(fs::remove_dir(filename));
    }

    #[test]
    fn file_test_fileinfo_false_when_checking_is_file_on_a_directory() {
        let tmpdir = tmpdir();
        let dir = &tmpdir.join("fileinfo_false_on_dir");
        check!(fs::create_dir(dir));
        assert!(!dir.is_file());
        check!(fs::remove_dir(dir));
    }

    #[test]
    fn file_test_fileinfo_check_exists_before_and_after_file_creation() {
        let tmpdir = tmpdir();
        let file = &tmpdir.join("fileinfo_check_exists_b_and_a.txt");
        check!(check!(File::create(file)).write(b"foo"));
        assert!(file.exists());
        check!(fs::remove_file(file));
        assert!(!file.exists());
    }

    #[test]
    fn file_test_directoryinfo_check_exists_before_and_after_mkdir() {
        let tmpdir = tmpdir();
        let dir = &tmpdir.join("before_and_after_dir");
        assert!(!dir.exists());
        check!(fs::create_dir(dir));
        assert!(dir.exists());
        assert!(dir.is_dir());
        check!(fs::remove_dir(dir));
        assert!(!dir.exists());
    }

    #[test]
    fn file_test_directoryinfo_readdir() {
        let tmpdir = tmpdir();
        let dir = &tmpdir.join("di_readdir");
        check!(fs::create_dir(dir));
        let prefix = "foo";
        for n in 0..3 {
            let f = dir.join(&format!("{}.txt", n));
            let mut w = check!(File::create(&f));
            let msg_str = format!("{}{}", prefix, n.to_string());
            let msg = msg_str.as_bytes();
            check!(w.write(msg));
        }
        let files = check!(fs::read_dir(dir));
        let mut mem = [0; 4];
        for f in files {
            let f = f.unwrap().path();
            {
                let n = f.file_stem().unwrap();
                check!(check!(File::open(&f)).read(&mut mem));
                let read_str = str::from_utf8(&mem).unwrap();
                let expected = format!("{}{}", prefix, n.to_str().unwrap());
                assert_eq!(expected, read_str);
            }
            check!(fs::remove_file(&f));
        }
        check!(fs::remove_dir(dir));
    }

    #[test]
    fn file_create_new_already_exists_error() {
        let tmpdir = tmpdir();
        let file = &tmpdir.join("file_create_new_error_exists");
        check!(fs::File::create(file));
        let e = fs::OpenOptions::new().write(true).create_new(true).open(file).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::AlreadyExists);
    }

    #[test]
    fn mkdir_path_already_exists_error() {
        let tmpdir = tmpdir();
        let dir = &tmpdir.join("mkdir_error_twice");
        check!(fs::create_dir(dir));
        let e = fs::create_dir(dir).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::AlreadyExists);
    }

    #[test]
    fn recursive_mkdir() {
        let tmpdir = tmpdir();
        let dir = tmpdir.join("d1/d2");
        check!(fs::create_dir_all(&dir));
        assert!(dir.is_dir())
    }

    #[test]
    fn recursive_mkdir_failure() {
        let tmpdir = tmpdir();
        let dir = tmpdir.join("d1");
        let file = dir.join("f1");

        check!(fs::create_dir_all(&dir));
        check!(File::create(&file));

        let result = fs::create_dir_all(&file);

        assert!(result.is_err());
    }

    #[test]
    fn recursive_mkdir_slash() {
        check!(fs::create_dir_all(&Path::new("/")));
    }

    #[test]
    fn recursive_rmdir() {
        let tmpdir = tmpdir();
        let d1 = tmpdir.join("d1");
        let dt = d1.join("t");
        let dtt = dt.join("t");
        let d2 = tmpdir.join("d2");
        let canary = d2.join("do_not_delete");
        check!(fs::create_dir_all(&dtt));
        check!(fs::create_dir_all(&d2));
        check!(check!(File::create(&canary)).write(b"foo"));
        check!(symlink_junction(&d2, &dt.join("d2")));
        let _ = symlink_file(&canary, &d1.join("canary"));
        check!(fs::remove_dir_all(&d1));

        assert!(!d1.is_dir());
        assert!(canary.exists());
    }

    #[test]
    fn recursive_rmdir_of_symlink() {
        // test we do not recursively delete a symlink but only dirs.
        let tmpdir = tmpdir();
        let link = tmpdir.join("d1");
        let dir = tmpdir.join("d2");
        let canary = dir.join("do_not_delete");
        check!(fs::create_dir_all(&dir));
        check!(check!(File::create(&canary)).write(b"foo"));
        check!(symlink_junction(&dir, &link));
        check!(fs::remove_dir_all(&link));

        assert!(!link.is_dir());
        assert!(canary.exists());
    }

    #[test]
    // only Windows makes a distinction between file and directory symlinks.
    #[cfg(windows)]
    fn recursive_rmdir_of_file_symlink() {
        let tmpdir = tmpdir();
        if !got_symlink_permission(&tmpdir) { return };

        let f1 = tmpdir.join("f1");
        let f2 = tmpdir.join("f2");
        check!(check!(File::create(&f1)).write(b"foo"));
        check!(symlink_file(&f1, &f2));
        match fs::remove_dir_all(&f2) {
            Ok(..) => panic!("wanted a failure"),
            Err(..) => {}
        }
    }

    #[test]
    fn unicode_path_is_dir() {
        assert!(Path::new(".").is_dir());
        assert!(!Path::new("test/stdtest/fs.rs").is_dir());

        let tmpdir = tmpdir();

        let mut dirpath = tmpdir.path().to_path_buf();
        dirpath.push("test-가一ー你好");
        check!(fs::create_dir(&dirpath));
        assert!(dirpath.is_dir());

        let mut filepath = dirpath;
        filepath.push("unicode-file-\u{ac00}\u{4e00}\u{30fc}\u{4f60}\u{597d}.rs");
        check!(File::create(&filepath)); // ignore return; touch only
        assert!(!filepath.is_dir());
        assert!(filepath.exists());
    }

    #[test]
    fn unicode_path_exists() {
        assert!(Path::new(".").exists());
        assert!(!Path::new("test/nonexistent-bogus-path").exists());

        let tmpdir = tmpdir();
        let unicode = tmpdir.path();
        let unicode = unicode.join(&format!("test-각丁ー再见"));
        check!(fs::create_dir(&unicode));
        assert!(unicode.exists());
        assert!(!Path::new("test/unicode-bogus-path-각丁ー再见").exists());
    }

    #[test]
    fn copy_file_does_not_exist() {
        let from = Path::new("test/nonexistent-bogus-path");
        let to = Path::new("test/other-bogus-path");

        match fs::copy(&from, &to) {
            Ok(..) => panic!(),
            Err(..) => {
                assert!(!from.exists());
                assert!(!to.exists());
            }
        }
    }

    #[test]
    fn copy_src_does_not_exist() {
        let tmpdir = tmpdir();
        let from = Path::new("test/nonexistent-bogus-path");
        let to = tmpdir.join("out.txt");
        check!(check!(File::create(&to)).write(b"hello"));
        assert!(fs::copy(&from, &to).is_err());
        assert!(!from.exists());
        let mut v = Vec::new();
        check!(check!(File::open(&to)).read_to_end(&mut v));
        assert_eq!(v, b"hello");
    }

    #[test]
    fn copy_file_ok() {
        let tmpdir = tmpdir();
        let input = tmpdir.join("in.txt");
        let out = tmpdir.join("out.txt");

        check!(check!(File::create(&input)).write(b"hello"));
        check!(fs::copy(&input, &out));
        let mut v = Vec::new();
        check!(check!(File::open(&out)).read_to_end(&mut v));
        assert_eq!(v, b"hello");

        assert_eq!(check!(input.metadata()).permissions(),
                   check!(out.metadata()).permissions());
    }

    #[test]
    fn copy_file_dst_dir() {
        let tmpdir = tmpdir();
        let out = tmpdir.join("out");

        check!(File::create(&out));
        match fs::copy(&*out, tmpdir.path()) {
            Ok(..) => panic!(), Err(..) => {}
        }
    }

    #[test]
    fn copy_file_dst_exists() {
        let tmpdir = tmpdir();
        let input = tmpdir.join("in");
        let output = tmpdir.join("out");

        check!(check!(File::create(&input)).write("foo".as_bytes()));
        check!(check!(File::create(&output)).write("bar".as_bytes()));
        check!(fs::copy(&input, &output));

        let mut v = Vec::new();
        check!(check!(File::open(&output)).read_to_end(&mut v));
        assert_eq!(v, b"foo".to_vec());
    }

    #[test]
    fn copy_file_src_dir() {
        let tmpdir = tmpdir();
        let out = tmpdir.join("out");

        match fs::copy(tmpdir.path(), &out) {
            Ok(..) => panic!(), Err(..) => {}
        }
        assert!(!out.exists());
    }

    #[test]
    fn copy_file_preserves_perm_bits() {
        let tmpdir = tmpdir();
        let input = tmpdir.join("in.txt");
        let out = tmpdir.join("out.txt");

        let attr = check!(check!(File::create(&input)).metadata());
        let mut p = attr.permissions();
        p.set_readonly(true);
        check!(fs::set_permissions(&input, p));
        check!(fs::copy(&input, &out));
        assert!(check!(out.metadata()).permissions().readonly());
        check!(fs::set_permissions(&input, attr.permissions()));
        check!(fs::set_permissions(&out, attr.permissions()));
    }

    #[test]
    #[cfg(windows)]
    fn copy_file_preserves_streams() {
        let tmp = tmpdir();
        check!(check!(File::create(tmp.join("in.txt:bunny"))).write("carrot".as_bytes()));
        assert_eq!(check!(fs::copy(tmp.join("in.txt"), tmp.join("out.txt"))), 6);
        assert_eq!(check!(tmp.join("out.txt").metadata()).len(), 0);
        let mut v = Vec::new();
        check!(check!(File::open(tmp.join("out.txt:bunny"))).read_to_end(&mut v));
        assert_eq!(v, b"carrot".to_vec());
    }

    #[test]
    fn symlinks_work() {
        let tmpdir = tmpdir();
        if !got_symlink_permission(&tmpdir) { return };

        let input = tmpdir.join("in.txt");
        let out = tmpdir.join("out.txt");

        check!(check!(File::create(&input)).write("foobar".as_bytes()));
        check!(symlink_file(&input, &out));
        assert!(check!(out.symlink_metadata()).file_type().is_symlink());
        assert_eq!(check!(fs::metadata(&out)).len(),
                   check!(fs::metadata(&input)).len());
        let mut v = Vec::new();
        check!(check!(File::open(&out)).read_to_end(&mut v));
        assert_eq!(v, b"foobar".to_vec());
    }

    #[test]
    fn symlink_noexist() {
        // Symlinks can point to things that don't exist
        let tmpdir = tmpdir();
        if !got_symlink_permission(&tmpdir) { return };

        // Use a relative path for testing. Symlinks get normalized by Windows,
        // so we may not get the same path back for absolute paths
        check!(symlink_file(&"foo", &tmpdir.join("bar")));
        assert_eq!(check!(fs::read_link(&tmpdir.join("bar"))).to_str().unwrap(),
                   "foo");
    }

    #[test]
    fn read_link() {
        if cfg!(windows) {
            // directory symlink
            assert_eq!(check!(fs::read_link(r"C:\Users\All Users")).to_str().unwrap(),
                       r"C:\ProgramData");
            // junction
            assert_eq!(check!(fs::read_link(r"C:\Users\Default User")).to_str().unwrap(),
                       r"C:\Users\Default");
            // junction with special permissions
            assert_eq!(check!(fs::read_link(r"C:\Documents and Settings\")).to_str().unwrap(),
                       r"C:\Users");
        }
        let tmpdir = tmpdir();
        let link = tmpdir.join("link");
        if !got_symlink_permission(&tmpdir) { return };
        check!(symlink_file(&"foo", &link));
        assert_eq!(check!(fs::read_link(&link)).to_str().unwrap(), "foo");
    }

    #[test]
    fn readlink_not_symlink() {
        let tmpdir = tmpdir();
        match fs::read_link(tmpdir.path()) {
            Ok(..) => panic!("wanted a failure"),
            Err(..) => {}
        }
    }

    #[test]
    fn links_work() {
        let tmpdir = tmpdir();
        let input = tmpdir.join("in.txt");
        let out = tmpdir.join("out.txt");

        check!(check!(File::create(&input)).write("foobar".as_bytes()));
        check!(fs::hard_link(&input, &out));
        assert_eq!(check!(fs::metadata(&out)).len(),
                   check!(fs::metadata(&input)).len());
        assert_eq!(check!(fs::metadata(&out)).len(),
                   check!(input.metadata()).len());
        let mut v = Vec::new();
        check!(check!(File::open(&out)).read_to_end(&mut v));
        assert_eq!(v, b"foobar".to_vec());

        // can't link to yourself
        match fs::hard_link(&input, &input) {
            Ok(..) => panic!("wanted a failure"),
            Err(..) => {}
        }
        // can't link to something that doesn't exist
        match fs::hard_link(&tmpdir.join("foo"), &tmpdir.join("bar")) {
            Ok(..) => panic!("wanted a failure"),
            Err(..) => {}
        }
    }

    #[test]
    fn chmod_works() {
        let tmpdir = tmpdir();
        let file = tmpdir.join("in.txt");

        check!(File::create(&file));
        let attr = check!(fs::metadata(&file));
        assert!(!attr.permissions().readonly());
        let mut p = attr.permissions();
        p.set_readonly(true);
        check!(fs::set_permissions(&file, p.clone()));
        let attr = check!(fs::metadata(&file));
        assert!(attr.permissions().readonly());

        match fs::set_permissions(&tmpdir.join("foo"), p.clone()) {
            Ok(..) => panic!("wanted an error"),
            Err(..) => {}
        }

        p.set_readonly(false);
        check!(fs::set_permissions(&file, p));
    }

    #[test]
    fn sync_doesnt_kill_anything() {
        let tmpdir = tmpdir();
        let path = tmpdir.join("in.txt");

        let mut file = check!(File::create(&path));
        check!(file.sync_all());
        check!(file.sync_data());
        check!(file.write(b"foo"));
        check!(file.sync_all());
        check!(file.sync_data());
    }

    #[test]
    fn truncate_works() {
        let tmpdir = tmpdir();
        let path = tmpdir.join("in.txt");

        let mut file = check!(File::create(&path));
        check!(file.write(b"foo"));
        check!(file.sync_all());

        // Do some simple things with truncation
        assert_eq!(check!(file.metadata()).len(), 3);
        check!(file.set_len(10));
        assert_eq!(check!(file.metadata()).len(), 10);
        check!(file.write(b"bar"));
        check!(file.sync_all());
        assert_eq!(check!(file.metadata()).len(), 10);

        let mut v = Vec::new();
        check!(check!(File::open(&path)).read_to_end(&mut v));
        assert_eq!(v, b"foobar\0\0\0\0".to_vec());

        // Truncate to a smaller length, don't seek, and then write something.
        // Ensure that the intermediate zeroes are all filled in (we have `seek`ed
        // past the end of the file).
        check!(file.set_len(2));
        assert_eq!(check!(file.metadata()).len(), 2);
        check!(file.write(b"wut"));
        check!(file.sync_all());
        assert_eq!(check!(file.metadata()).len(), 9);
        let mut v = Vec::new();
        check!(check!(File::open(&path)).read_to_end(&mut v));
        assert_eq!(v, b"fo\0\0\0\0wut".to_vec());
    }

    #[test]
    fn open_flavors() {
        use fs::OpenOptions as OO;
        fn c<T: Clone>(t: &T) -> T { t.clone() }

        let tmpdir = tmpdir();

        let mut r = OO::new(); r.read(true);
        let mut w = OO::new(); w.write(true);
        let mut rw = OO::new(); rw.read(true).write(true);
        let mut a = OO::new(); a.append(true);
        let mut ra = OO::new(); ra.read(true).append(true);

        let invalid_options = if cfg!(windows) { "The parameter is incorrect" }
                              else { "Invalid argument" };

        // Test various combinations of creation modes and access modes.
        //
        // Allowed:
        // creation mode           | read  | write | read-write | append | read-append |
        // :-----------------------|:-----:|:-----:|:----------:|:------:|:-----------:|
        // not set (open existing) |   X   |   X   |     X      |   X    |      X      |
        // create                  |       |   X   |     X      |   X    |      X      |
        // truncate                |       |   X   |     X      |        |             |
        // create and truncate     |       |   X   |     X      |        |             |
        // create_new              |       |   X   |     X      |   X    |      X      |
        //
        // tested in reverse order, so 'create_new' creates the file, and 'open existing' opens it.

        // write-only
        check!(c(&w).create_new(true).open(&tmpdir.join("a")));
        check!(c(&w).create(true).truncate(true).open(&tmpdir.join("a")));
        check!(c(&w).truncate(true).open(&tmpdir.join("a")));
        check!(c(&w).create(true).open(&tmpdir.join("a")));
        check!(c(&w).open(&tmpdir.join("a")));

        // read-only
        error!(c(&r).create_new(true).open(&tmpdir.join("b")), invalid_options);
        error!(c(&r).create(true).truncate(true).open(&tmpdir.join("b")), invalid_options);
        error!(c(&r).truncate(true).open(&tmpdir.join("b")), invalid_options);
        error!(c(&r).create(true).open(&tmpdir.join("b")), invalid_options);
        check!(c(&r).open(&tmpdir.join("a"))); // try opening the file created with write_only

        // read-write
        check!(c(&rw).create_new(true).open(&tmpdir.join("c")));
        check!(c(&rw).create(true).truncate(true).open(&tmpdir.join("c")));
        check!(c(&rw).truncate(true).open(&tmpdir.join("c")));
        check!(c(&rw).create(true).open(&tmpdir.join("c")));
        check!(c(&rw).open(&tmpdir.join("c")));

        // append
        check!(c(&a).create_new(true).open(&tmpdir.join("d")));
        error!(c(&a).create(true).truncate(true).open(&tmpdir.join("d")), invalid_options);
        error!(c(&a).truncate(true).open(&tmpdir.join("d")), invalid_options);
        check!(c(&a).create(true).open(&tmpdir.join("d")));
        check!(c(&a).open(&tmpdir.join("d")));

        // read-append
        check!(c(&ra).create_new(true).open(&tmpdir.join("e")));
        error!(c(&ra).create(true).truncate(true).open(&tmpdir.join("e")), invalid_options);
        error!(c(&ra).truncate(true).open(&tmpdir.join("e")), invalid_options);
        check!(c(&ra).create(true).open(&tmpdir.join("e")));
        check!(c(&ra).open(&tmpdir.join("e")));

        // Test opening a file without setting an access mode
        let mut blank = OO::new();
         error!(blank.create(true).open(&tmpdir.join("f")), invalid_options);

        // Test write works
        check!(check!(File::create(&tmpdir.join("h"))).write("foobar".as_bytes()));

        // Test write fails for read-only
        check!(r.open(&tmpdir.join("h")));
        {
            let mut f = check!(r.open(&tmpdir.join("h")));
            assert!(f.write("wut".as_bytes()).is_err());
        }

        // Test write overwrites
        {
            let mut f = check!(c(&w).open(&tmpdir.join("h")));
            check!(f.write("baz".as_bytes()));
        }
        {
            let mut f = check!(c(&r).open(&tmpdir.join("h")));
            let mut b = vec![0; 6];
            check!(f.read(&mut b));
            assert_eq!(b, "bazbar".as_bytes());
        }

        // Test truncate works
        {
            let mut f = check!(c(&w).truncate(true).open(&tmpdir.join("h")));
            check!(f.write("foo".as_bytes()));
        }
        assert_eq!(check!(fs::metadata(&tmpdir.join("h"))).len(), 3);

        // Test append works
        assert_eq!(check!(fs::metadata(&tmpdir.join("h"))).len(), 3);
        {
            let mut f = check!(c(&a).open(&tmpdir.join("h")));
            check!(f.write("bar".as_bytes()));
        }
        assert_eq!(check!(fs::metadata(&tmpdir.join("h"))).len(), 6);

        // Test .append(true) equals .write(true).append(true)
        {
            let mut f = check!(c(&w).append(true).open(&tmpdir.join("h")));
            check!(f.write("baz".as_bytes()));
        }
        assert_eq!(check!(fs::metadata(&tmpdir.join("h"))).len(), 9);
    }

    #[test]
    fn _assert_send_sync() {
        fn _assert_send_sync<T: Send + Sync>() {}
        _assert_send_sync::<OpenOptions>();
    }

    #[test]
    fn binary_file() {
        let mut bytes = [0; 1024];
        StdRng::new().unwrap().fill_bytes(&mut bytes);

        let tmpdir = tmpdir();

        check!(check!(File::create(&tmpdir.join("test"))).write(&bytes));
        let mut v = Vec::new();
        check!(check!(File::open(&tmpdir.join("test"))).read_to_end(&mut v));
        assert!(v == &bytes[..]);
    }

    #[test]
    fn file_try_clone() {
        let tmpdir = tmpdir();

        let mut f1 = check!(OpenOptions::new()
                                       .read(true)
                                       .write(true)
                                       .create(true)
                                       .open(&tmpdir.join("test")));
        let mut f2 = check!(f1.try_clone());

        check!(f1.write_all(b"hello world"));
        check!(f1.seek(SeekFrom::Start(2)));

        let mut buf = vec![];
        check!(f2.read_to_end(&mut buf));
        assert_eq!(buf, b"llo world");
        drop(f2);

        check!(f1.write_all(b"!"));
    }

    #[test]
    #[cfg(not(windows))]
    fn unlink_readonly() {
        let tmpdir = tmpdir();
        let path = tmpdir.join("file");
        check!(File::create(&path));
        let mut perm = check!(fs::metadata(&path)).permissions();
        perm.set_readonly(true);
        check!(fs::set_permissions(&path, perm));
        check!(fs::remove_file(&path));
    }

    #[test]
    fn mkdir_trailing_slash() {
        let tmpdir = tmpdir();
        let path = tmpdir.join("file");
        check!(fs::create_dir_all(&path.join("a/")));
    }

    #[test]
    fn canonicalize_works_simple() {
        let tmpdir = tmpdir();
        let tmpdir = fs::canonicalize(tmpdir.path()).unwrap();
        let file = tmpdir.join("test");
        File::create(&file).unwrap();
        assert_eq!(fs::canonicalize(&file).unwrap(), file);
    }

    #[test]
    fn realpath_works() {
        let tmpdir = tmpdir();
        if !got_symlink_permission(&tmpdir) { return };

        let tmpdir = fs::canonicalize(tmpdir.path()).unwrap();
        let file = tmpdir.join("test");
        let dir = tmpdir.join("test2");
        let link = dir.join("link");
        let linkdir = tmpdir.join("test3");

        File::create(&file).unwrap();
        fs::create_dir(&dir).unwrap();
        symlink_file(&file, &link).unwrap();
        symlink_dir(&dir, &linkdir).unwrap();

        assert!(link.symlink_metadata().unwrap().file_type().is_symlink());

        assert_eq!(fs::canonicalize(&tmpdir).unwrap(), tmpdir);
        assert_eq!(fs::canonicalize(&file).unwrap(), file);
        assert_eq!(fs::canonicalize(&link).unwrap(), file);
        assert_eq!(fs::canonicalize(&linkdir).unwrap(), dir);
        assert_eq!(fs::canonicalize(&linkdir.join("link")).unwrap(), file);
    }

    #[test]
    fn realpath_works_tricky() {
        let tmpdir = tmpdir();
        if !got_symlink_permission(&tmpdir) { return };

        let tmpdir = fs::canonicalize(tmpdir.path()).unwrap();
        let a = tmpdir.join("a");
        let b = a.join("b");
        let c = b.join("c");
        let d = a.join("d");
        let e = d.join("e");
        let f = a.join("f");

        fs::create_dir_all(&b).unwrap();
        fs::create_dir_all(&d).unwrap();
        File::create(&f).unwrap();
        if cfg!(not(windows)) {
            symlink_dir("../d/e", &c).unwrap();
            symlink_file("../f", &e).unwrap();
        }
        if cfg!(windows) {
            symlink_dir(r"..\d\e", &c).unwrap();
            symlink_file(r"..\f", &e).unwrap();
        }

        assert_eq!(fs::canonicalize(&c).unwrap(), f);
        assert_eq!(fs::canonicalize(&e).unwrap(), f);
    }

    #[test]
    fn dir_entry_methods() {
        let tmpdir = tmpdir();

        fs::create_dir_all(&tmpdir.join("a")).unwrap();
        File::create(&tmpdir.join("b")).unwrap();

        for file in tmpdir.path().read_dir().unwrap().map(|f| f.unwrap()) {
            let fname = file.file_name();
            match fname.to_str() {
                Some("a") => {
                    assert!(file.file_type().unwrap().is_dir());
                    assert!(file.metadata().unwrap().is_dir());
                }
                Some("b") => {
                    assert!(file.file_type().unwrap().is_file());
                    assert!(file.metadata().unwrap().is_file());
                }
                f => panic!("unknown file name: {:?}", f),
            }
        }
    }

    #[test]
    fn dir_entry_debug() {
        let tmpdir = tmpdir();
        File::create(&tmpdir.join("b")).unwrap();
        let mut read_dir = tmpdir.path().read_dir().unwrap();
        let dir_entry = read_dir.next().unwrap().unwrap();
        let actual = format!("{:?}", dir_entry);
        let expected = format!("DirEntry({:?})", dir_entry.0.path());
        assert_eq!(actual, expected);
    }

    #[test]
    fn read_dir_not_found() {
        let res = fs::read_dir("/path/that/does/not/exist");
        assert_eq!(res.err().unwrap().kind(), ErrorKind::NotFound);
    }

    #[test]
    fn create_dir_all_with_junctions() {
        let tmpdir = tmpdir();
        let target = tmpdir.join("target");

        let junction = tmpdir.join("junction");
        let b = junction.join("a/b");

        let link = tmpdir.join("link");
        let d = link.join("c/d");

        fs::create_dir(&target).unwrap();

        check!(symlink_junction(&target, &junction));
        check!(fs::create_dir_all(&b));
        // the junction itself is not a directory, but `is_dir()` on a Path
        // follows links
        assert!(junction.is_dir());
        assert!(b.exists());

        if !got_symlink_permission(&tmpdir) { return };
        check!(symlink_dir(&target, &link));
        check!(fs::create_dir_all(&d));
        assert!(link.is_dir());
        assert!(d.exists());
    }

    #[test]
    fn metadata_access_times() {
        let tmpdir = tmpdir();

        let b = tmpdir.join("b");
        File::create(&b).unwrap();

        let a = check!(fs::metadata(&tmpdir.path()));
        let b = check!(fs::metadata(&b));

        assert_eq!(check!(a.accessed()), check!(a.accessed()));
        assert_eq!(check!(a.modified()), check!(a.modified()));
        assert_eq!(check!(b.accessed()), check!(b.modified()));

        if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
            check!(a.created());
            check!(b.created());
        }
    }
}
