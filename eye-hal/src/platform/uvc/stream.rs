use std::sync::{mpsc, Arc};

use crate::error::Result;
use crate::platform::uvc::device::UvcHandle;
use crate::traits::Stream;

pub struct Handle<'a> {
    rx: mpsc::Receiver<uvc::Result<uvc::Frame>>,

    // these are required to keep the frame callback alive
    _stream: uvc::ActiveStream<'a, mpsc::SyncSender<uvc::Result<uvc::Frame>>>,
    _stream_handle: uvc::StreamHandle<'a>,
    _dev_handle: Arc<UvcHandle<'a>>,
}

impl<'a> Handle<'a> {
    pub fn new(
        dev_handle: Arc<UvcHandle<'a>>,
        mut stream_handle: uvc::StreamHandle<'a>,
    ) -> uvc::Result<Self> {
        let stream_handle_ptr = &mut stream_handle as *mut uvc::StreamHandle;
        let stream_handle_ref = unsafe { &mut *stream_handle_ptr as &mut uvc::StreamHandle };

        // establish a rendezvous channel
        let (tx, rx) = mpsc::sync_channel(0);
        let stream = stream_handle_ref.start_stream(
            |frame, tx| {
                match tx.send(frame.to_rgb()) {
                    Ok(()) => {}
                    Err(_) => {
                        // The receiving end hung up.
                        // This should only ever happen once (when self.rx is dropped).
                    }
                }
            },
            tx,
        )?;

        Ok(Handle {
            rx,
            _stream: stream,
            _stream_handle: stream_handle,
            _dev_handle: dev_handle,
        })
    }
}

impl<'a, 'b> Stream<'b> for Handle<'a> {
    type Item = Result<&'b [u8]>;

    fn next(&'b mut self) -> Option<Self::Item> {
        let frame = self.rx.recv().unwrap();
        let pixels = match &frame {
            Ok(frame) => frame.to_bytes(),
            Err(_) => {
                // The format conversion failed. Pretend the stream died.
                return None;
            }
        };

        Some(Ok(&pixels))
    }
}
