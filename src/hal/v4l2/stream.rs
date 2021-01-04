use std::{io, mem};

use v4l::buffer::Type as BufType;
use v4l::io::mmap::Stream as MmapStream;
use v4l::io::traits::{CaptureStream, Stream};
use v4l::video::Capture;

use crate::format::{Format, FourCC, PixelFormat};
use crate::hal::v4l2::device::PlatformDevice;
use crate::image::CowImage;
use crate::traits::Stream as StreamTrait;

pub struct PlatformStream<'a> {
    format: Format,
    stream: MmapStream<'a>,
    stream_buf_index: usize,
    active: bool,
}

impl<'a> PlatformStream<'a> {
    pub fn new(dev: &PlatformDevice) -> io::Result<Self> {
        let format_ = dev.inner().format()?;
        let format = Format::with_stride(
            format_.width,
            format_.height,
            PixelFormat::from(FourCC::new(&format_.fourcc.repr)),
            format_.stride as usize,
        );

        let stream = MmapStream::new(dev.inner(), BufType::VideoCapture)?;
        Ok(PlatformStream {
            format,
            stream,
            stream_buf_index: 0,
            active: false,
        })
    }

    fn start(&mut self) -> io::Result<()> {
        if self.active {
            return Ok(());
        }

        self.stream.start()?;
        self.active = true;
        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        self.stream.stop()?;
        self.active = false;
        Ok(())
    }

    fn queue(&mut self) -> io::Result<()> {
        if !self.active {
            self.start()?;
        }

        self.stream.queue(self.stream_buf_index)?;
        Ok(())
    }

    fn dequeue<'b>(&'b mut self) -> io::Result<CowImage<'b>> {
        self.stream_buf_index = self.stream.dequeue()?;

        let buf = self.stream.get(self.stream_buf_index).unwrap();
        let image = CowImage::from_slice(buf, self.format);

        // The Rust compiler thinks we're returning a value (view) which references data owned by
        // the local function (frame). This is actually not the case since the data slice is
        // memory mapped and thus the actual backing memory resides somewhere else
        // (kernel, on-chip, etc).
        unsafe { Ok(mem::transmute(image)) }
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

impl<'a, 'b> StreamTrait<'b> for PlatformStream<'a> {
    type Item = io::Result<CowImage<'b>>;

    fn next(&'b mut self) -> Option<Self::Item> {
        if let Err(e) = self.queue() {
            return Some(Err(e));
        }

        Some(self.dequeue())
    }
}
