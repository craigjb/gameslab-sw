extern crate memmap;
extern crate png;

use std::path::Path;
use std::fs::{OpenOptions, File};

const SCREEN_W: usize = 800;
const SCREEN_H: usize = 480;

#[derive(Clone, Copy)]
struct RGB {
    r: u8,
    g: u8,
    b: u8
}

trait Pixel: Clone + Copy {
    fn bytes() -> usize;
    fn from_bytes(bytes: &[u8]) -> Self;
    fn write_bytes(&self, dest: &mut[u8]);
}

impl Pixel for RGB {
    fn bytes() -> usize {
        3
    }

    fn from_bytes(bytes: &[u8]) -> RGB {
        RGB {
            r: bytes[0],
            g: bytes[1],
            b: bytes[2]
        }
    }

    fn write_bytes(&self, dest: &mut[u8]) {
        dest[0] = self.r;
        dest[1] = self.g;
        dest[2] = self.b;
    }
}

impl Pixel for u8 {
    fn bytes() -> usize {
        1
    }

    fn from_bytes(bytes: &[u8]) -> u8 {
        bytes[0]
    }

    fn write_bytes(&self, dest: &mut[u8]) {
        dest[0] = *self
    }
}

struct Image {
    width: usize,
    height: usize,
    data: Vec<u8>
}

impl Image {
    pub fn new<P: Pixel>(width: usize, height: usize) -> Image {
        Image {
            width: width,
            height: height,
            data: vec![0; width * height * P::bytes()]
        }
    }

    pub fn load_png<PT: AsRef<Path>>(path: PT) -> Result<Image, png::DecodingError> {
        let decoder = png::Decoder::new(File::open(path)?);
        let (info, mut reader) = decoder.read_info()?;
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf)?;

        Ok(Image {
            width: info.width as usize,
            height: info.height as usize,
            data: buf
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn get_pixel<P: Pixel>(&self, x: usize, y: usize) -> P {
        let offset = (self.width * y * P::bytes()) + (x * P::bytes());
        P::from_bytes(&self.data[offset..])
    }

    pub fn get_float(&self, x: usize, y: usize) -> f32 {
        let offset = (self.width * y) + x;
        (self.data[offset] as f32) / 255.0
    }

    pub fn set_pixel<P: Pixel>(&mut self, x: usize, y: usize, c: P) {
        let offset = (self.width * y * P::bytes()) + (x * P::bytes());
        c.write_bytes(&mut self.data[offset..]);
    }

    pub fn draw<P: Pixel>(&mut self, img: &Image,
                         sx: usize, sy: usize, width: usize, height: usize,
                         dx: usize, dy: usize) {
        for y in 0..height {
            for x in 0..width {
                self.set_pixel::<P>(dx + x, dy + y, img.get_pixel::<P>(sx + x, sy + y));
            }
        }
    }
}

struct FrameBuffer {
    mmap: memmap::MmapMut,
    width: usize,
    height: usize
}

impl FrameBuffer {
    pub fn from_fbdev<P: AsRef<Path>>(path: P, width: usize, height: usize) ->Result<FrameBuffer, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        let mut map = unsafe {
            memmap::MmapOptions::new()
                .len(width * height * 3)
                .map_mut(&file)?
        };
        
        Ok(FrameBuffer {
            mmap: map,
            width: width,
            height: height
        })
    }

    pub fn display(&mut self, image: &Image) {
        self.mmap.copy_from_slice(image.data());
        self.mmap.flush();
    }
}

fn draw_vertical_line(dest: &mut Image, x: i32, height: i32, c: RGB) {
    for y in (height as usize)..dest.height() {
        dest.set_pixel(x as usize, y, c);
    }
}

fn draw_terrain(dest: &mut Image, hmap: &Image, cmap: &Image,
                p: (f32, f32, f32), horizon: f32, zscale: f32, distance: f32) {
    let mut z = distance;
    while z > 1.0 {
        let mut current = (-z + p.0, -z + p.1);
        let pright = (z + p.0, -z + p.1);
        let dx = (pright.0 - current.0) / (dest.width() as f32);

        for i in 0..dest.width() {
            let color = cmap.get_pixel::<RGB>(current.0 as usize + 512, current.1 as usize + 512);
            let height = hmap.get_float(current.0 as usize + 512, current.1 as usize + 512);
            let height_on_screen = ((p.2 - (height * zscale)) / z) + horizon;
            draw_vertical_line(dest, i as i32, height_on_screen as i32, color);
            current = (current.0 + dx, current.1);
        }

        z -= 1.0;
    }
}

fn main() {
    let mut framebuffer = FrameBuffer::from_fbdev("/dev/fb0", SCREEN_W, SCREEN_H).unwrap();
    let mut backbuffer = Image::new::<RGB>(SCREEN_W, SCREEN_H);
    let color_map = Image::load_png("C28W.png").unwrap();
    let height_map = Image::load_png("D25.png").unwrap();

    draw_terrain(&mut backbuffer, &height_map, &color_map, (0.0, 0.0, 50.0), 120.0, 120.0, 100.0);

    framebuffer.display(&backbuffer);
}
