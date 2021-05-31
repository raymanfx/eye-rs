use ffimage::color::{Bgr, Rgb};
use ffimage::packed::{ImageBuffer, ImageView};
use ffimage::traits::{Convert, Pixel};

use eye_hal::format::{ImageFormat, PixelFormat};

fn _convert<DP>(
    src: &[u8],
    src_fmt: &ImageFormat,
    dst: &mut ImageBuffer<DP>,
) -> Result<(), &'static str>
where
    DP: Pixel + From<Rgb<u8>> + Copy + Send,
    DP::T: Default + Clone + Send,
{
    let rgb = match ImageView::<Rgb<u8>>::from_buf(src, src_fmt.width, src_fmt.height) {
        Some(view) => view,
        None => return Err("failed to create RGB view"),
    };

    rgb.convert(dst);
    Ok(())
}

pub fn convert_to_bgr(
    src: &[u8],
    src_fmt: &ImageFormat,
    dst: &mut Vec<u8>,
) -> Result<(), &'static str> {
    let mut bgr = ImageBuffer::<Bgr<u8>>::new(src_fmt.width, src_fmt.height, 0u8);
    match _convert(src, src_fmt, &mut bgr) {
        Ok(()) => {
            *dst = bgr.into_buf();
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn convert(
    src: &[u8],
    src_fmt: &ImageFormat,
    dst: &mut Vec<u8>,
    dst_fmt: &PixelFormat,
) -> Result<(), &'static str> {
    match dst_fmt {
        PixelFormat::Bgr(24) => convert_to_bgr(src, src_fmt, dst),
        _ => Err("cannot handle target format"),
    }
}
