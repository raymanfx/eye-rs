use std::borrow::Cow;

/// Buffer abstraction
///
/// TODO: multiple planes
#[derive(Clone)]
pub struct Buffer<'a> {
    inner: Cow<'a, [u8]>,
}

impl<'a> Buffer<'a> {
    /// Returns the raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        match &self.inner {
            Cow::Borrowed(slice) => slice,
            Cow::Owned(buf) => buf,
        }
    }

    /// Returns the raw bytes
    pub fn into_bytes(self) -> impl Iterator<Item = u8> {
        match self.inner {
            Cow::Borrowed(slice) => slice.to_vec().into_iter(),
            Cow::Owned(buf) => buf.into_iter(),
        }
    }

    /// Returns an instance that is guaranteed to own its data
    ///
    /// If the instance currently borrows the data, it is cloned and transferred. Otherwise, no
    /// allocation is needed and the owned data is reused.
    pub fn own<'b>(self) -> Buffer<'b> {
        Buffer {
            inner: Cow::Owned(self.inner.into_owned()),
        }
    }
}

impl<'a> From<&'a [u8]> for Buffer<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Buffer {
            inner: Cow::Borrowed(bytes),
        }
    }
}

impl<'a> From<Vec<u8>> for Buffer<'a> {
    fn from(bytes: Vec<u8>) -> Self {
        Buffer {
            inner: Cow::Owned(bytes),
        }
    }
}
