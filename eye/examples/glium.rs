use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use eye::colorconvert::Device;
use eye_hal::format::PixelFormat;
use eye_hal::traits::{Context as _, Device as _, Stream as _};
use eye_hal::{Error, ErrorKind, PlatformContext, Result};

use glium::index::PrimitiveType;
use glium::{glutin, Surface};
use glium::{implement_vertex, program, uniform};

fn main() -> Result<()> {
    // Create a context
    let ctx = if let Some(ctx) = PlatformContext::all().next() {
        ctx
    } else {
        return Err(Error::new(
            ErrorKind::Other,
            "No platform context available",
        ));
    };

    // Create a list of valid capture devices in the system.
    let dev_descrs = ctx.devices()?;

    // Print the supported formats for each device.
    let dev = ctx.open_device(&dev_descrs[0].uri)?;
    let dev = Device::new(dev)?;
    let stream_descr = dev
        .streams()?
        .into_iter()
        .reduce(|s1, s2| {
            // Choose RGB with 8 bit depth
            if s1.pixfmt == PixelFormat::Rgb(24) && s2.pixfmt != PixelFormat::Rgb(24) {
                return s1;
            }

            // Strive for HD (1280 x 720)
            let distance = |width: u32, height: u32| {
                f32::sqrt(((1280 - width as i32).pow(2) + (720 - height as i32).pow(2)) as f32)
            };

            if distance(s1.width, s1.height) < distance(s2.width, s2.height) {
                s1
            } else {
                s2
            }
        })
        .unwrap();

    if stream_descr.pixfmt != PixelFormat::Rgb(24) {
        return Err(Error::new(ErrorKind::Other, "No RGB3 streams available"));
    }

    println!("Selected stream:\n{:?}", stream_descr);

    // Start the stream
    let mut stream = dev.start_stream(&stream_descr)?;

    // Setup the GL display stuff
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }

        implement_vertex!(Vertex, position, tex_coords);

        glium::VertexBuffer::new(
            &display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
        )
        .unwrap()
    };

    // building the index buffer
    let index_buffer =
        glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip, &[1u16, 2, 0, 3]).unwrap();

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                in vec2 tex_coords;
                out vec2 v_tex_coords;
                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                out vec4 f_color;
                void main() {
                    f_color = texture(tex, v_tex_coords);
                }
            "
        },
    )
    .unwrap();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || loop {
        let buf = stream.next().unwrap().unwrap();
        tx.send(buf.to_vec()).unwrap();
    });

    event_loop.run(move |event, _, control_flow| {
        let t0 = Instant::now();
        let buf = rx.recv().unwrap();
        let t1 = Instant::now();

        let image = glium::texture::RawImage2d::from_raw_rgb_reversed(
            &buf,
            (stream_descr.width, stream_descr.height),
        );
        let opengl_texture = glium::texture::Texture2d::new(&display, image).unwrap();

        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex: &opengl_texture
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();

        // polling and handling the events received by the window
        if let glutin::event::Event::WindowEvent {
            event: glutin::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = glutin::event_loop::ControlFlow::Exit;
        }

        print!(
            "\rms: {}\t (buffer) + {}\t (UI)",
            t1.duration_since(t0).as_millis(),
            t1.elapsed().as_millis()
        );
    });
}
