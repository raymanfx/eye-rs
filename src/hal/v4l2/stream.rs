use std::{io, mem};

use ffimage::packed::DynamicImageView;

use v4l::buffer::{Buffer, BufferStream};
use v4l::MappedBufferStream;

use crate::format::{Format, FourCC};
use crate::hal::traits::{Stream, StreamItem};
use crate::hal::v4l2::device::PlatformDevice;

pub struct PlatformStream<'a> {
    format: Format,
    stream: Option<MappedBufferStream<'a>>,
    active: bool,
    queued: bool,
}

impl<'a> PlatformStream<'a> {
    pub fn new(dev: &'a mut PlatformDevice) -> io::Result<Self> {
        let format_ = dev.inner().get_format()?;
        let format = Format::with_stride(
            format_.width,
            format_.height,
            FourCC::new(&format_.fourcc.repr),
            format_.stride as usize,
        );

        let mut stream = PlatformStream {
            format,
            stream: None,
            active: false,

            // the v4l2 backend queues a number of buffers by default, so skip the first queue()
            // call for this high-level interface
            queued: true,
        };
        stream.stream = Some(MappedBufferStream::new(dev.inner_mut())?);
        Ok(stream)
    }

    fn start(&mut self) -> io::Result<()> {
        if self.active {
            return Ok(());
        }

        self.stream.as_mut().unwrap().start()?;
        self.active = true;
        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        self.stream.as_mut().unwrap().stop()?;
        self.active = false;
        Ok(())
    }

    fn queue(&mut self) -> io::Result<()> {
        if !self.active {
            self.start()?;
        }

        if self.queued {
            return Ok(());
        }

        self.stream.as_mut().unwrap().queue()?;
        self.queued = true;
        Ok(())
    }

    fn dequeue(&mut self) -> io::Result<DynamicImageView<'a>> {
        let frame = self.stream.as_mut().unwrap().dequeue()?;
        self.queued = false;

        let view = DynamicImageView::with_stride(
            frame.data(),
            self.format.width,
            self.format.height,
            self.format.stride.unwrap_or(0),
        )
        .unwrap();

        // The Rust compiler thinks we're returning a value (view) which references data owned by
        // the local function (frame). This is actually not the case since the data slice is
        // memory mapped and thus the actual backing memory resides somewhere else
        // (kernel, on-chip, etc).
        unsafe { Ok(mem::transmute(view)) }
    }
}

impl<'a> Drop for PlatformStream<'a> {
    fn drop(&mut self) {
        if self.active {
            // ignore the result
            let _ = self.stop();
        }
    }
}

impl<'a> Stream for PlatformStream<'a> {
    type Item = DynamicImageView<'a>;

    fn next<'b>(&'b mut self) -> io::Result<StreamItem<'b, Self::Item>> {
        self.queue()?;
        let item = self.dequeue()?;

        Ok(StreamItem::new(item))
    }
}
