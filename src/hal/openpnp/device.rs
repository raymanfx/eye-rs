use std::io;

use openpnp_capture as pnp;

use ffimage::packed::dynamic::ImageView;

use crate::control;
use crate::format::{Format, FourCC, PixelFormat};
use crate::hal::openpnp::stream::PlatformStream;
use crate::hal::traits::{Device, Stream};

pub struct PlatformDevice {
    inner: pnp::Device,
    pub(crate) format: Option<pnp::Format>,
}

impl PlatformDevice {
    pub fn new(index: u32) -> Option<Self> {
        let dev = PlatformDevice {
            inner: pnp::Device::new(index)?,
            format: None,
        };
        Some(dev)
    }

    pub fn inner(&self) -> &pnp::Device {
        &self.inner
    }
}

impl Device for PlatformDevice {
    fn query_formats(&self) -> io::Result<Vec<Format>> {
        let plat_formats = self.inner.formats();
        let formats: Vec<Format> = plat_formats
            .into_iter()
            .map(|fmt| Format {
                width: fmt.width,
                height: fmt.height,
                pixfmt: PixelFormat::from(FourCC::new(&fmt.fourcc.repr)),
                stride: None,
            })
            .collect();

        // openpnp only ever outputs RGB24
        let mut found_rgb24 = false;
        for fmt in &formats {
            if fmt.pixfmt == PixelFormat::Rgb(24) {
                found_rgb24 = true;
                break;
            }
        }

        let mut rgb_formats: Vec<Format> = Vec::new();
        if found_rgb24 {
            rgb_formats = formats;
        } else {
            // add all the resolutions, but make RGB24 the only pixelformat
            for fmt in &formats {
                let mut found = false;
                for rgb_fmt in &rgb_formats {
                    if rgb_fmt.width == fmt.width && rgb_fmt.height == fmt.height {
                        found = true;
                        break;
                    }
                }

                if !found {
                    rgb_formats.push(Format {
                        width: fmt.width,
                        height: fmt.height,
                        pixfmt: PixelFormat::Rgb(24),
                        stride: None,
                    })
                }
            }
        }

        Ok(rgb_formats)
    }

    fn query_controls(&self) -> io::Result<Vec<control::Control>> {
        Err(io::Error::new(io::ErrorKind::Other, "not supported"))
    }

    fn control(&self, _id: u32) -> io::Result<control::Value> {
        Err(io::Error::new(io::ErrorKind::Other, "not supported"))
    }

    fn set_control(&mut self, _id: u32, _val: &control::Value) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Other, "not supported"))
    }

    fn format(&self) -> io::Result<Format> {
        if let Some(format) = self.format {
            return Ok(Format {
                width: format.width,
                height: format.height,
                pixfmt: PixelFormat::Rgb(24),
                stride: None,
            });
        }

        let formats = self.query_formats()?;
        if formats.len() > 0 {
            let mut format = formats[0];
            format.pixfmt = PixelFormat::Rgb(24);
            Ok(format)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "no formats available"))
        }
    }

    fn set_format(&mut self, fmt: &Format) -> io::Result<Format> {
        if fmt.pixfmt != PixelFormat::Rgb(24) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "only RGB24 is supported",
            ));
        }

        self.format = Some(pnp::Format {
            width: fmt.width,
            height: fmt.height,
            fourcc: pnp::format::FourCC::new(b"RGB3"),
            bpp: 0,
            fps: 0,
        });

        // TODO: return the actual format
        Ok(*fmt)
    }

    fn stream<'a>(&self) -> io::Result<Box<dyn 'a + for<'b> Stream<'b, Item = ImageView<'b>>>> {
        let stream = PlatformStream::new(self)?;
        Ok(Box::new(stream))
    }
}
