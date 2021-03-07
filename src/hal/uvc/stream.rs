use std::borrow::Cow;
use std::io;
use std::sync::{mpsc, Arc};

use crate::format::{ImageFormat, PixelFormat};
use crate::frame::Frame;
use crate::hal::uvc::device::UvcHandle;
use crate::traits::Stream;

pub struct Handle<'a> {
    _stream: uvc::ActiveStream<'a, mpsc::SyncSender<uvc::Result<uvc::Frame>>>,
    _stream_handle: uvc::StreamHandle<'a>,
    _dev_handle: Arc<UvcHandle<'a>>,
    format: uvc::StreamFormat,
    rx: mpsc::Receiver<uvc::Result<uvc::Frame>>,
}

impl<'a> Handle<'a> {
    pub fn new(
        dev_handle: Arc<UvcHandle<'a>>,
        mut stream_handle: uvc::StreamHandle<'a>,
        format: uvc::StreamFormat,
    ) -> uvc::Result<Self> {
        let stream_handle_ptr = &mut stream_handle as *mut uvc::StreamHandle;
        let stream_handle_ref = unsafe { &mut *stream_handle_ptr as &mut uvc::StreamHandle };

        // establish a rendezvous channel
        let (tx, rx) = mpsc::sync_channel(0);
        let stream =
            stream_handle_ref.start_stream(|frame, tx| tx.send(frame.to_rgb()).unwrap(), tx)?;

        Ok(Handle {
            _stream: stream,
            _stream_handle: stream_handle,
            _dev_handle: dev_handle,
            format,
            rx,
        })
    }
}

impl<'a, 'b> Stream<'b> for Handle<'a> {
    type Item = io::Result<Frame<'b>>;

    fn next(&'b mut self) -> Option<Self::Item> {
        let frame = self.rx.recv().unwrap();
        let pixels = match &frame {
            Ok(frame) => frame.to_bytes(),
            Err(_) => {
                return None;
            }
        };

        let buffer = Cow::Owned(pixels.to_vec());
        let format = ImageFormat::new(self.format.width, self.format.height, PixelFormat::Rgb(24));

        Some(Ok(Frame { buffer, format }))
    }
}
