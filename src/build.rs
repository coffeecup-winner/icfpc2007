use std::cell::Cell;
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Position(pub u32, pub u32);

impl Position {
    pub fn move_(&self, dir: Direction) -> Self {
        let Position(x, y) = *self;
        use Direction::*;
        match dir {
            Up => Position(x, y.wrapping_sub(1) % 600),
            Right => Position(x.wrapping_add(1) % 600, y),
            Down => Position(x, y.wrapping_add(1) % 600),
            Left => Position(x.wrapping_sub(1) % 600, y),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct RGB(pub u8, pub u8, pub u8);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Pixel {
    pub rgb: RGB,
    pub a: u8,
}

#[derive(Clone)]
pub struct Bitmap {
    data: Vec<Pixel>,
}

impl Bitmap {
    pub fn transparent() -> Self {
        Bitmap {
            data: vec![Pixel {
                rgb: BLACK,
                a: TRANSPARENT,
            }; 600 * 600],
        }
    }

    pub fn get(&self, Position(x, y): Position) -> Pixel {
        self.data[(y * 600 + x) as usize]
    }

    pub fn set(&mut self, Position(x, y): Position, pixel: Pixel) {
        self.data[(y * 600 + x) as usize] = pixel;
    }

    pub fn draw_line(
        &mut self,
        Position(x0, y0): Position,
        Position(x1, y1): Position,
        pixel: Pixel,
    ) {
        let dx = x1 as f32 - x0 as f32;
        let dy = y1 as f32 - y0 as f32;
        let d = dx.abs().max(dy.abs()) as u32;
        let c = if dx * dy <= 0f32 { 1 } else { 0 } as u32;
        let mut x = (x0 * d + ((d - c) / 2)) as f32;
        let mut y = (y0 * d + ((d - c) / 2)) as f32;
        for _ in 0..d {
            self.set(
                Position((x / d as f32) as u32, (y / d as f32) as u32),
                pixel,
            );
            x += dx;
            y += dy;
        }
        self.set(Position(x1, y1), pixel);
    }

    pub fn fill(&mut self, Position(x, y): Position, new: Pixel) {
        let old = self.get(Position(x, y));
        if old != new {
            let mut queue = VecDeque::new();
            queue.push_back(Position(x, y));
            while let Some(p) = queue.pop_front() {
                self.set(p, new);
                if x > 0 {
                    let left = p.move_(Direction::Left);
                    if self.get(left) == old {
                        queue.push_back(left);
                    }
                }
                if x < 599 {
                    let right = p.move_(Direction::Right);
                    if self.get(right) == old {
                        queue.push_back(right);
                    }
                }
                if y > 0 {
                    let up = p.move_(Direction::Up);
                    if self.get(up) == old {
                        queue.push_back(up);
                    }
                }
                if y < 599 {
                    let down = p.move_(Direction::Down);
                    if self.get(down) == old {
                        queue.push_back(down);
                    }
                }
            }
        }
    }

    pub fn compose_with(&mut self, other: Self) {
        for y in 0..600u32 {
            for x in 0..600u32 {
                let Pixel {
                    rgb: RGB(r0, g0, b0),
                    a: a0,
                } = other.get(Position(x, y));
                let Pixel {
                    rgb: RGB(r1, g1, b1),
                    a: a1,
                } = self.get(Position(x, y));
                self.set(
                    Position(x, y),
                    Pixel {
                        rgb: RGB(
                            r0 + (r1 * (255 - a0) / 255),
                            g0 + (g1 * (255 - a0) / 255),
                            b0 + (b1 * (255 - a0) / 255),
                        ),
                        a: a0 + (a1 * (255 - a0) / 255),
                    },
                )
            }
        }
    }

    pub fn clip_with(&mut self, other: Self) {
        for y in 0..600u32 {
            for x in 0..600u32 {
                let Pixel { rgb: _, a: a0 } = other.get(Position(x, y));
                let Pixel {
                    rgb: RGB(r1, g1, b1),
                    a: a1,
                } = self.get(Position(x, y));
                self.set(
                    Position(x, y),
                    Pixel {
                        rgb: RGB(r1 * a0 / 255, g1 * a0 / 255, b1 * a0 / 255),
                        a: a1 * a0 / 255,
                    },
                )
            }
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Color {
    RGB(RGB),
    Transparency(u8),
}
struct Bucket {
    bucket: Vec<Color>,
    current: Cell<Option<Pixel>>,
}

impl Bucket {
    pub fn new() -> Self {
        Bucket {
            bucket: vec![],
            current: Cell::new(None),
        }
    }

    pub fn clear(&mut self) {
        self.bucket.clear();
        *self.current.get_mut() = None;
    }

    pub fn add_color(&mut self, color: Color) {
        self.bucket.push(color);
        *self.current.get_mut() = None;
    }

    pub fn current_pixel(&self) -> Pixel {
        if let Some(pixel) = self.current.get() {
            return pixel;
        }
        let mut rs = vec![];
        let mut gs = vec![];
        let mut bs = vec![];
        let mut as_ = vec![];
        for &c in self.bucket.iter() {
            match c {
                Color::RGB(RGB(r, g, b)) => {
                    rs.push(r);
                    gs.push(g);
                    bs.push(b);
                }
                Color::Transparency(a) => {
                    as_.push(a);
                }
            }
        }
        let r = Self::average(rs, 0);
        let g = Self::average(gs, 0);
        let b = Self::average(bs, 0);
        let a = Self::average(as_, 255);
        let pixel = Pixel {
            rgb: RGB(
                (r * a / 255) as u8,
                (g * a / 255) as u8,
                (b * a / 255) as u8,
            ),
            a: a as u8,
        };
        self.current.set(Some(pixel));
        pixel
    }

    fn average(values: Vec<u8>, default: u32) -> u32 {
        if values.is_empty() {
            default
        } else {
            let len = values.len() as u32;
            values.into_iter().map(|c| c as u32).sum::<u32>() / len
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn turn_ccw(&self) -> Self {
        use Direction::*;
        match self {
            Up => Left,
            Right => Up,
            Down => Right,
            Left => Down,
        }
    }

    pub fn turn_cw(&self) -> Self {
        use Direction::*;
        match self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up,
        }
    }
}

const BLACK: RGB = RGB(0, 0, 0);
const RED: RGB = RGB(255, 0, 0);
const GREEN: RGB = RGB(0, 255, 0);
const YELLOW: RGB = RGB(255, 255, 0);
const BLUE: RGB = RGB(0, 0, 255);
const MAGENTA: RGB = RGB(255, 0, 255);
const CYAN: RGB = RGB(0, 255, 255);
const WHITE: RGB = RGB(255, 255, 255);

const TRANSPARENT: u8 = 0;
const OPAQUE: u8 = 255;

pub fn build(mut rna: &[u8]) -> Bitmap {
    let mut bucket = Bucket::new();
    let mut pos = Position(0, 0);
    let mut mark = Position(0, 0);
    let mut dir = Direction::Right;
    let mut bitmaps = vec![Bitmap::transparent()];
    while rna.len() >= 7 {
        match &rna[0..7] {
            b"PIPIIIC" => bucket.add_color(Color::RGB(BLACK)),
            b"PIPIIIP" => bucket.add_color(Color::RGB(RED)),
            b"PIPIICC" => bucket.add_color(Color::RGB(GREEN)),
            b"PIPIICF" => bucket.add_color(Color::RGB(YELLOW)),
            b"PIPIICP" => bucket.add_color(Color::RGB(BLUE)),
            b"PIPIIFC" => bucket.add_color(Color::RGB(MAGENTA)),
            b"PIPIIFF" => bucket.add_color(Color::RGB(CYAN)),
            b"PIPIIPC" => bucket.add_color(Color::RGB(WHITE)),
            b"PIPIIPF" => bucket.add_color(Color::Transparency(TRANSPARENT)),
            b"PIPIIPP" => bucket.add_color(Color::Transparency(OPAQUE)),
            b"PIIPICP" => bucket.clear(),
            b"PIIIIIP" => pos = pos.move_(dir),
            b"PCCCCCP" => dir = dir.turn_ccw(),
            b"PFFFFFP" => dir = dir.turn_cw(),
            b"PCCIFFP" => mark = pos,
            b"PFFICCP" => {
                let idx = bitmaps.len() - 1;
                bitmaps[idx].draw_line(pos, mark, bucket.current_pixel());
            }
            b"PIIPIIP" => {
                let idx = bitmaps.len() - 1;
                bitmaps[idx].fill(pos, bucket.current_pixel());
            }
            b"PCCPFFP" => bitmaps.push(Bitmap::transparent()),
            b"PFFPCCP" => {
                if bitmaps.len() > 1 {
                    let bitmap = bitmaps.pop().unwrap();
                    let idx = bitmaps.len() - 1;
                    bitmaps[idx].compose_with(bitmap);
                }
            }
            b"PFFICCF" => {
                if bitmaps.len() > 1 {
                    let bitmap = bitmaps.pop().unwrap();
                    let idx = bitmaps.len() - 1;
                    bitmaps[idx].clip_with(bitmap);
                }
            }
            _ => {}
        }
        rna = &rna[8..];
    }
    bitmaps.pop().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket() {
        let b = Color::RGB(BLACK);
        let r = Color::RGB(RED);
        let m = Color::RGB(MAGENTA);
        let w = Color::RGB(WHITE);
        let y = Color::RGB(YELLOW);
        let c = Color::RGB(CYAN);
        let t = Color::Transparency(TRANSPARENT);
        let o = Color::Transparency(OPAQUE);

        let add_colors = |bucket: &mut Bucket, colors: Vec<Color>| {
            for c in colors {
                bucket.add_color(c);
            }
        };

        let mut bucket = Bucket::new();
        add_colors(&mut bucket, vec![o, o, t]);
        assert_eq!(
            bucket.current_pixel(),
            Pixel {
                rgb: RGB(0, 0, 0),
                a: 170
            }
        );

        bucket.clear();
        add_colors(&mut bucket, vec![c, y, b]);
        assert_eq!(
            bucket.current_pixel(),
            Pixel {
                rgb: RGB(85, 170, 85),
                a: 255
            }
        );

        bucket.clear();
        add_colors(&mut bucket, vec![o, t, y]);
        assert_eq!(
            bucket.current_pixel(),
            Pixel {
                rgb: RGB(127, 127, 0),
                a: 127
            }
        );

        bucket.clear();
        let mut colors = vec![];
        colors.extend(&[t; 1]);
        colors.extend(&[o; 3]);
        colors.extend(&[w; 10]);
        colors.extend(&[m; 32]); // rust doesn't support automatic traits on arrays with >32 elements
        colors.extend(&[m; 7]); // so split &[m; 39] in two
        colors.extend(&[r; 7]);
        colors.extend(&[b; 18]);
        add_colors(&mut bucket, colors);
        assert_eq!(
            bucket.current_pixel(),
            Pixel {
                rgb: RGB(143, 25, 125),
                a: 191
            }
        );
    }
}
