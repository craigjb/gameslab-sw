extern crate memmap;
extern crate png;
extern crate time;

use std::path::Path;
use std::fs::{OpenOptions, File};
use std::io::BufWriter;
use png::HasParameters;
use time::PreciseTime;

const SCREEN_W: i32 = 800;
const SCREEN_H: i32 = 480;

fn load_png<P: AsRef<Path>>(path: P) -> Result<(i32, i32, Vec<u8>), png::DecodingError> {
    let decoder = png::Decoder::new(File::open(path)?);
    let (info, mut reader) = decoder.read_info()?;
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf)?;

    Ok((info.width as i32, info.height as i32, buf))
}

fn save_png<P: AsRef<Path>>(path: P, width: i32, height: i32, data: &[u8]) -> Result<(), png::EncodingError> {
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width as u32, height as u32);
    encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;
    Ok(())
}

struct HeightMap {
    width: i32,
    height: i32,
    data: Vec<u8>
}

impl HeightMap {
    pub fn from_png<P: AsRef<Path>>(path: P) -> Result<HeightMap, png::DecodingError> {
        let (width, height, data) = load_png(path)?;
        Ok(HeightMap {
            width: width,
            height: height,
            data: data
        })
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn get_height(&self, x: i32, y: i32) -> f32 {
        let offset = ((self.width * y) + x) as usize;
        (self.data[offset] as f32) / 255.0
    }
}

struct Image {
    width: i32,
    height: i32,
    data: Vec<u8>
}

impl Image {
    pub fn new(width: i32, height: i32) -> Image {
        Image {
            width: width,
            height: height,
            data: vec![0; (width * height * 3) as usize]
        }
    }

    pub fn from_png<P: AsRef<Path>>(path: P) -> Result<Image, png::DecodingError> {
        let (width, height, data) = load_png(path)?;
        Ok(Image {
            width: width,
            height: height,
            data: data
        })
    }

    pub fn save_png<P: AsRef<Path>>(&self, path: P) -> Result<(), png::EncodingError> {
        save_png(path, self.width, self.height, self.data.as_slice())?;
        Ok(())
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> (u8, u8, u8) {
        let offset = ((self.width * y * 3) + (x * 3)) as usize;
        (self.data[offset], self.data[offset + 1], self.data[offset + 2])
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, c: (u8, u8, u8)) {
        let offset = ((self.width * y * 3) + (x * 3)) as usize;
        self.data[offset] = c.0;
        self.data[offset + 1] = c.1;
        self.data[offset + 2] = c.2;
    }

    pub fn fill_rect(&mut self, x: i32, y:i32, width: i32, height: i32, c: (u8, u8, u8)) {
        for iy in 0..height {
            for ix in 0..width {
                self.set_pixel(x + ix, y + iy, c)
            }
        }
    }

    pub fn draw_vertical_line(&mut self, x: i32, y0: i32, y1: i32, c: (u8, u8, u8)) {
        let mut offset = ((self.width * y0 * 3) + (x * 3)) as usize;
        let stride = (self.width * 3) as usize;
        for _ in y0..y1 {
            self.data[offset] = c.0;
            self.data[offset + 1] = c.1;
            self.data[offset + 2] = c.2;
            offset += stride;
        }
    }

    pub fn draw(&mut self, img: &Image,
                sx: i32, sy: i32, width: i32, height: i32,
                dx: i32, dy: i32) {
        for y in 0..height {
            for x in 0..width {
                self.set_pixel(dx + x, dy + y, img.get_pixel(sx + x, sy + y));
            }
        }
    }
}

struct FrameBuffer {
    mmap: memmap::MmapMut,
    width: i32,
    height: i32
}

impl FrameBuffer {
    pub fn from_fbdev<P: AsRef<Path>>(path: P, width: i32, height: i32) -> Result<FrameBuffer, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        let mut map = unsafe {
            memmap::MmapOptions::new()
                .len((width * height * 3) as usize)
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

fn draw_terrain(dest: &mut Image, hmap: &HeightMap, cmap: &Image,
                p: (f32, f32, f32), hmap_scale: f32,  horizon: f32, zscale: f32, distance: f32) {
    let dest_width = dest.width();
    let dest_height = dest.height();

    let mut ybuffer = vec![dest_height; dest_width as usize];

    let mut z = 1.0;
    while z < distance {
        let mut current = (-z + p.0, -z + p.1);
        let pright = (z + p.0, -z + p.1);
        let dx = (pright.0 - current.0) / (dest.width() as f32);

        for i in 0..dest.width() {
            let xi = current.0 as i32 + 512;
            let yi = current.1 as i32 + 512;
            if xi < 0 || xi >= 1024 || yi < 0 || yi >= 1024 {
                continue;
            }

            let height = hmap.get_height(xi, yi) * hmap_scale;
            let height_on_screen = (((p.2 - height) / z * zscale) + horizon) as i32;
            if height_on_screen < dest_height {
                let color = cmap.get_pixel(xi, yi);
                dest.draw_vertical_line(i as i32, height_on_screen as i32, ybuffer[i as usize], color);
            }
            if height_on_screen < ybuffer[i as usize] {
                ybuffer[i as usize] = height_on_screen;
            }
            current = (current.0 + dx, current.1);
        }

        z += 1.0;
    }
}

fn main() {
    let mut framebuffer = FrameBuffer::from_fbdev("/dev/fb0", SCREEN_W, SCREEN_H).unwrap();
    let mut backbuffer = Image::new(SCREEN_W, SCREEN_H);
    let height_map = HeightMap::from_png("D25.png").unwrap();
    let color_map = Image::from_png("C28W.png").unwrap();

    let before = PreciseTime::now();
    for _ in 0..60 {
        backbuffer.fill_rect(0, 0, SCREEN_W, SCREEN_H, (51, 204, 255));
        draw_terrain(&mut backbuffer, &height_map, &color_map, (190.0, 190.0, 200.0), 200.0, 100.0, 120.0, 300.0);
        framebuffer.display(&backbuffer);
    }
    let after = PreciseTime::now();
    println!("Profile: {:?}", before.to(after).num_milliseconds());

    //backbuffer.save_png("out.png").unwrap();
}
