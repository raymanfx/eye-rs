use std::{io, mem};

use ffimage::packed::image::dynamic::{MemoryView, StorageType};
use ffimage::packed::{DynamicImageBuffer, DynamicImageView};

use crate::format::{Format, PixelFormat};
use crate::hal::common::convert::Converter;
use crate::hal::traits::{Stream, StreamItem};

/// A transparent wrapper for native platform streams.
pub struct TransparentStream<'a> {
    stream: Box<dyn Stream<Item = DynamicImageView<'a>> + 'a>,
    format: Format,
    mapping: Option<(PixelFormat, PixelFormat)>,
    convert_buffer: DynamicImageBuffer,
}

impl<'a> TransparentStream<'a> {
    pub fn new(stream: Box<dyn Stream<Item = DynamicImageView<'a>> + 'a>, format: Format) -> Self {
        TransparentStream {
            stream,
            format,
            mapping: None,
            convert_buffer: DynamicImageBuffer::empty(StorageType::U8),
        }
    }

    /// Tell the stream to emulate a format by requesting the compatible source format from the
    /// native device.
    ///
    /// # Arguments
    ///
    /// * `src` - Source format to be set on the device
    /// * `dst` - Target format to emulate
    ///
    pub fn map(&mut self, src: PixelFormat, dst: PixelFormat) {
        self.mapping = Some((src, dst))
    }
}

impl<'a> Stream for TransparentStream<'a> {
    type Item = DynamicImageView<'a>;

    fn next<'b>(&'b mut self) -> io::Result<StreamItem<'b, Self::Item>> {
        let mut view = self.stream.next()?;

        // emulate format by converting the buffer if necessary
        if let Some(map) = self.mapping {
            match view.raw() {
                MemoryView::U8(_) => {
                    self.convert_buffer = DynamicImageBuffer::empty(StorageType::U8);
                }
                MemoryView::U16(_) => {
                    self.convert_buffer = DynamicImageBuffer::empty(StorageType::U16);
                }
            }

            Converter::convert(&view, self.format.pixfmt, &mut self.convert_buffer, map.1)?;
            view = StreamItem::new(self.convert_buffer.as_view())
        }

        // The Rust compiler thinks we're returning a value (view) which references data owned by
        // the local function (frame). This is actually not the case since the data slice is
        // memory mapped and thus the actual backing memory resides somewhere else
        // (kernel, on-chip, etc).
        unsafe { Ok(mem::transmute(view)) }
    }
}
