use v4l::buffer::Type as BufType;
use v4l::io::mmap::Stream as MmapStream;
use v4l::io::traits::{CaptureStream, Stream as _};

use crate::error::Result;
use crate::platform::v4l2::device::Handle as DeviceHandle;
use crate::traits::Stream;

pub struct Handle<'a> {
    stream: MmapStream<'a>,
    active: bool,
}

impl<'a> Handle<'a> {
    pub fn new(dev: &DeviceHandle) -> Result<Self> {
        let stream = MmapStream::new(dev.inner(), BufType::VideoCapture)?;
        Ok(Handle {
            stream,
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
        match self.stream.next() {
            Ok((buf, meta)) => {
                // For compressed formats, the buffer length will not actually describe the number
                // of bytes in a frame. Instead, we have to explicitly query about the amount of
                // used bytes.
                Some(Ok(&buf[0..meta.bytesused as usize]))
            }
            Err(e) => Some(Err(e.into())),
        }
    }
}
