use v4l::buffer::Type as BufType;
use v4l::io::mmap::Stream as MmapStream;
use v4l::io::traits::{CaptureStream, Stream as _};

use crate::error::Result;
use crate::platform::v4l2::device::Handle as DeviceHandle;
use crate::traits::Stream;

pub struct Handle<'a> {
    stream: MmapStream<'a>,
    stream_buf_index: usize,
    active: bool,
}

impl<'a> Handle<'a> {
    pub fn new(dev: &DeviceHandle) -> Result<Self> {
        let stream = MmapStream::new(dev.inner(), BufType::VideoCapture)?;
        Ok(Handle {
            stream,
            stream_buf_index: 0,
            active: false,
        })
    }

    fn start(&mut self) -> Result<()> {
        if self.active {
            return Ok(());
        }

        self.stream.start()?;
        self.active = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        self.stream.stop()?;
        self.active = false;
        Ok(())
    }

    fn queue(&mut self) -> Result<()> {
        if !self.active {
            self.start()?;
        }

        self.stream.queue(self.stream_buf_index)?;
        Ok(())
    }

    fn dequeue(&mut self) -> Result<&[u8]> {
        self.stream_buf_index = self.stream.dequeue()?;

        let buffer = self.stream.get(self.stream_buf_index).unwrap();
        let meta = self.stream.get_meta(self.stream_buf_index).unwrap();

        // For compressed formats, the buffer length will not actually describe the number of bytes
        // in a frame. Instead, we have to explicitly query about the amount of used bytes.
        let view = &buffer[0..meta.bytesused as usize];
        Ok(view)
    }
}

impl<'a> Drop for Handle<'a> {
    fn drop(&mut self) {
        if self.active {
            // ignore the result
            let _ = self.stop();
        }
    }
}

impl<'a, 'b> Stream<'b> for Handle<'a> {
    type Item = Result<&'b [u8]>;

    fn next(&'b mut self) -> Option<Self::Item> {
        if let Err(e) = self.queue() {
            return Some(Err(e));
        }

        Some(self.dequeue())
    }
}
