use ffimage::packed::dynamic::{ImageBuffer, ImageView, MemoryBuffer, MemoryView};

#[derive(Debug, Clone)]
enum Cow<'a> {
    Borrowed(ImageView<'a>),
    Owned(ImageBuffer),
}

#[derive(Debug)]
pub struct CowImage<'a> {
    data: Cow<'a>,
}

impl<'a> CowImage<'a> {
    /// Returns the raw pixel data
    pub fn raw(&self) -> MemoryView {
        match &self.data {
            Cow::Borrowed(view) => *view.raw(),
            Cow::Owned(buf) => match buf.raw() {
                MemoryBuffer::U8(data) => MemoryView::U8(data),
                MemoryBuffer::U16(data) => MemoryView::U16(data),
            },
        }
    }

    /// Returns the width in pixels
    pub fn width(&self) -> u32 {
        match &self.data {
            Cow::Borrowed(view) => view.width(),
            Cow::Owned(buf) => buf.width(),
        }
    }

    /// Returns the height in pixels
    pub fn height(&self) -> u32 {
        match &self.data {
            Cow::Borrowed(view) => view.height(),
            Cow::Owned(buf) => buf.height(),
        }
    }

    /// Returns the amount of bytes per pixel row
    pub fn stride(&self) -> usize {
        match &self.data {
            Cow::Borrowed(view) => view.stride(),
            Cow::Owned(buf) => buf.stride(),
        }
    }

    /// Returns a view into the image
    pub fn as_view(&self) -> ImageView {
        match &self.data {
            Cow::Borrowed(view) => *view,
            Cow::Owned(buf) => buf.as_view(),
        }
    }

    /// Returns a mutable representation of the image
    ///
    /// If the image data is borrowed, it is now cloned and owned by this instance. Otherwise, the
    /// already owned buffer is returned.
    pub fn to_mut(&mut self) -> &mut ImageBuffer {
        if let Cow::Borrowed(view) = &self.data {
            self.data = Cow::Owned(ImageBuffer::from(view));
        }

        match &mut self.data {
            Cow::Owned(buf) => buf,
            _ => panic!("impossible"),
        }
    }

    /// Returns an instance that is guaranteed to own its data
    ///
    /// If the instance currently borrows the data, it is cloned and transferred. Otherwise, no
    /// allocation is needed and the owned data is reused.
    pub fn own<'b>(self) -> CowImage<'b> {
        let buf = match self.data {
            Cow::Borrowed(view) => ImageBuffer::from(&view),
            Cow::Owned(buf) => buf,
        };

        CowImage {
            data: Cow::Owned(buf),
        }
    }

    /// Returns a deep copy of the contained data
    pub fn clone<'b>(&self) -> CowImage<'b> {
        match &self.data {
            Cow::Borrowed(view) => CowImage {
                data: Cow::Owned(ImageBuffer::from(view)),
            },
            Cow::Owned(buf) => CowImage {
                data: Cow::Owned(buf.clone()),
            },
        }
    }
}

impl<'a> Clone for CowImage<'a> {
    fn clone(&self) -> Self {
        self.clone()
    }
}

impl<'a> From<ImageView<'a>> for CowImage<'a> {
    fn from(view: ImageView<'a>) -> Self {
        CowImage {
            data: Cow::Borrowed(view),
        }
    }
}

impl<'a> From<ImageBuffer> for CowImage<'a> {
    fn from(buf: ImageBuffer) -> Self {
        CowImage {
            data: Cow::Owned(buf),
        }
    }
}

impl<'a> Into<ImageBuffer> for CowImage<'a> {
    fn into(self) -> ImageBuffer {
        match self.data {
            Cow::Borrowed(view) => ImageBuffer::from(&view),
            Cow::Owned(buf) => buf,
        }
    }
}
