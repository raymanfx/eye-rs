use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use eye::prelude::*;

fn main() {
    // Query for available devices.
    let devices = Context::enumerate_devices();
    if devices.is_empty() {
        println!("No devices available");
        return;
    }

    // Create a communication channel.
    let (tx, rx): (Sender<_>, Receiver<_>) = mpsc::channel();

    let capture_thread = thread::spawn(move || {
        // Create a device instance.
        let dev = Device::with_uri(&devices[0]).expect("Failed to open video device");

        // Create a stream to capture frames from.
        let mut stream = dev.stream().expect("Failed to setup capture stream");

        for _ in 0..10 {
            // Capture a frame.
            // Most of the time, this will be a view to the actual frame data which may be backed
            // by device drivers (e.g. via mmap) or others.
            let img = stream
                .next()
                .expect("Stream is dead")
                .expect("Failed to capture frame");
            // Sending a frame to another thread requires performing a deep clone of the original
            // data. This way, we can ensure that the frame data stays valid once the next frame is
            // deqeued from the stream.
            tx.send(img.own()).unwrap();
            println!("TX: sent image");
        }
    });

    for _ in 0..10 {
        // Receive a frame from the streaming thread.
        let _ = rx.recv().unwrap();
        println!("RX: got image");
    }

    capture_thread.join().unwrap();
}
