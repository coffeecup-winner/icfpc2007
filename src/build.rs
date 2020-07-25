use std::cell::Cell;
use std::collections::{HashSet, VecDeque};

use crate::types::*;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Position(pub u32, pub u32);

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

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
            data: vec![
                Pixel {
                    rgb: BLACK,
                    a: TRANSPARENT,
                };
                600 * 600
            ],
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
        let dx = x1 as i32 - x0 as i32;
        let dy = y1 as i32 - y0 as i32;
        let d = dx.abs().max(dy.abs()) as u32;
        let c = if dx * dy <= 0 { 1 } else { 0 } as u32;
        let mut x = (x0 * d + ((d - c) / 2)) as i32;
        let mut y = (y0 * d + ((d - c) / 2)) as i32;
        for _ in 0..d {
            self.set(
                Position((x / d as i32) as u32, (y / d as i32) as u32),
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
            let mut visited = HashSet::new();
            queue.push_back(Position(x, y));
            visited.insert(Position(x, y));
            while let Some(p) = queue.pop_front() {
                self.set(p, new);
                if x > 0 {
                    let left = p.move_(Direction::Left);
                    if !visited.contains(&left) && self.get(left) == old {
                        queue.push_back(left);
                        visited.insert(left);
                    }
                }
                if x < 599 {
                    let right = p.move_(Direction::Right);
                    if !visited.contains(&right) && self.get(right) == old {
                        queue.push_back(right);
                        visited.insert(right);
                    }
                }
                if y > 0 {
                    let up = p.move_(Direction::Up);
                    if !visited.contains(&up) && self.get(up) == old {
                        queue.push_back(up);
                        visited.insert(up);
                    }
                }
                if y < 599 {
                    let down = p.move_(Direction::Down);
                    if !visited.contains(&down) && self.get(down) == old {
                        queue.push_back(down);
                        visited.insert(down);
                    }
                }
                visited.insert(Position(x, y));
            }
        }
    }

    pub fn compose_with(&mut self, other: Self) {
        for y in 0..600u32 {
            for x in 0..600u32 {
                let p0 = other.get(Position(x, y));
                let r0 = p0.rgb.0 as u32;
                let g0 = p0.rgb.1 as u32;
                let b0 = p0.rgb.2 as u32;
                let a0 = p0.a as u32;
                let p1 = self.get(Position(x, y));
                let r1 = p1.rgb.0 as u32;
                let g1 = p1.rgb.1 as u32;
                let b1 = p1.rgb.2 as u32;
                let a1 = p1.a as u32;
                self.set(
                    Position(x, y),
                    Pixel {
                        rgb: RGB(
                            (r0 + (r1 * (255 - a0) / 255)) as u8,
                            (g0 + (g1 * (255 - a0) / 255)) as u8,
                            (b0 + (b1 * (255 - a0) / 255)) as u8,
                        ),
                        a: (a0 + (a1 * (255 - a0) / 255)) as u8,
                    },
                )
            }
        }
    }

    pub fn clip_with(&mut self, other: Self) {
        for y in 0..600u32 {
            for x in 0..600u32 {
                let p0 = other.get(Position(x, y));
                let a0 = p0.a as u32;
                let p1 = self.get(Position(x, y));
                let r1 = p1.rgb.0 as u32;
                let g1 = p1.rgb.1 as u32;
                let b1 = p1.rgb.2 as u32;
                let a1 = p1.a as u32;
                self.set(
                    Position(x, y),
                    Pixel {
                        rgb: RGB(
                            (r1 * a0 / 255) as u8,
                            (g1 * a0 / 255) as u8,
                            (b1 * a0 / 255) as u8,
                        ),
                        a: (a1 * a0 / 255) as u8,
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

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    AddBlack,
    AddRed,
    AddGreen,
    AddYellow,
    AddBlue,
    AddMagenta,
    AddCyan,
    AddWhite,
    AddTransparent,
    AddOpaque,
    ClearBucket,
    Move,
    TurnCcw,
    TurnCw,
    Mark,
    DrawLine,
    Fill,
    AddLayer,
    Compose,
    Clip,
    Unknown(Vec<Base>),
}

pub fn build(rna: &[u8]) -> Bitmap {
    let mut builder = BuilderState::new(&to_base_vec(rna));
    for _ in 0..builder.commands.len() {
        builder.step();
    }
    builder.bitmaps.pop().unwrap()
}

pub struct BuilderState {
    bucket: Bucket,
    pos: Position,
    mark: Position,
    dir: Direction,
    pub bitmaps: Vec<Bitmap>,
    pub commands: Vec<Command>,
    pub iteration: u32,
    pub enable_debug_prints: bool,
}

impl BuilderState {
    pub fn new(rna: &[Base]) -> Self {
        BuilderState {
            bucket: Bucket::new(),
            pos: Position(0, 0),
            mark: Position(0, 0),
            dir: Direction::Right,
            bitmaps: vec![Bitmap::transparent()],
            commands: Self::convert_rna_to_commands(rna),
            iteration: 0,
            enable_debug_prints: false,
        }
    }

    pub fn extend(&mut self, rna: &[Base]) {
        self.commands.extend(Self::convert_rna_to_commands(rna));
    }

    fn convert_rna_to_commands(mut rna: &[Base]) -> Vec<Command> {
        let mut commands = vec![];
        while rna.len() >= 7 {
            commands.push(match &rna[0..7] {
                &[P, I, P, I, I, I, C] => Command::AddBlack,
                &[P, I, P, I, I, I, P] => Command::AddRed,
                &[P, I, P, I, I, C, C] => Command::AddGreen,
                &[P, I, P, I, I, C, F] => Command::AddYellow,
                &[P, I, P, I, I, C, P] => Command::AddBlue,
                &[P, I, P, I, I, F, C] => Command::AddMagenta,
                &[P, I, P, I, I, F, F] => Command::AddCyan,
                &[P, I, P, I, I, P, C] => Command::AddWhite,
                &[P, I, P, I, I, P, F] => Command::AddTransparent,
                &[P, I, P, I, I, P, P] => Command::AddOpaque,
                &[P, I, I, P, I, C, P] => Command::ClearBucket,
                &[P, I, I, I, I, I, P] => Command::Move,
                &[P, C, C, C, C, C, P] => Command::TurnCcw,
                &[P, F, F, F, F, F, P] => Command::TurnCw,
                &[P, C, C, I, F, F, P] => Command::Mark,
                &[P, F, F, I, C, C, P] => Command::DrawLine,
                &[P, I, I, P, I, I, P] => Command::Fill,
                &[P, C, C, P, F, F, P] => Command::AddLayer,
                &[P, F, F, P, C, C, P] => Command::Compose,
                &[P, F, F, I, C, C, F] => Command::Clip,
                b => Command::Unknown(b.iter().cloned().collect()),
            });
            rna = &rna[7..];
        }
        commands
    }

    pub fn step(&mut self) -> &Bitmap {
        if self.enable_debug_prints {
            println!("Step {}", self.iteration);
        }
        match &self.commands[self.iteration as usize] {
            Command::AddBlack => {
                if self.enable_debug_prints {
                    println!("+BLACK");
                }
                self.bucket.add_color(Color::RGB(BLACK));
            }
            Command::AddRed => {
                if self.enable_debug_prints {
                    println!("+RED");
                }
                self.bucket.add_color(Color::RGB(RED));
            }
            Command::AddGreen => {
                if self.enable_debug_prints {
                    println!("+GREEN");
                }
                self.bucket.add_color(Color::RGB(GREEN));
            }
            Command::AddYellow => {
                if self.enable_debug_prints {
                    println!("+YELLOW");
                }
                self.bucket.add_color(Color::RGB(YELLOW));
            }
            Command::AddBlue => {
                if self.enable_debug_prints {
                    println!("+BLUE");
                }
                self.bucket.add_color(Color::RGB(BLUE));
            }
            Command::AddMagenta => {
                if self.enable_debug_prints {
                    println!("+MAGENTA");
                }
                self.bucket.add_color(Color::RGB(MAGENTA));
            }
            Command::AddCyan => {
                if self.enable_debug_prints {
                    println!("+CYAN");
                }
                self.bucket.add_color(Color::RGB(CYAN));
            }
            Command::AddWhite => {
                if self.enable_debug_prints {
                    println!("+WHITE");
                }
                self.bucket.add_color(Color::RGB(WHITE));
            }
            Command::AddTransparent => {
                if self.enable_debug_prints {
                    println!("+TRANSPARENT");
                }
                self.bucket.add_color(Color::Transparency(TRANSPARENT));
            }
            Command::AddOpaque => {
                if self.enable_debug_prints {
                    println!("+OPAQUE");
                }
                self.bucket.add_color(Color::Transparency(OPAQUE));
            }
            Command::ClearBucket => {
                if self.enable_debug_prints {
                    println!("BUCKET CLEAR");
                }
                self.bucket.clear();
            }
            Command::Move => {
                if self.enable_debug_prints {
                    print!("pos: {} -> ", self.pos);
                }
                self.pos = self.pos.move_(self.dir);
                if self.enable_debug_prints {
                    println!("{}", self.pos);
                }
            }
            Command::TurnCcw => {
                if self.enable_debug_prints {
                    print!("dir: {:?} -> ", self.dir);
                }
                self.dir = self.dir.turn_ccw();
                if self.enable_debug_prints {
                    println!("{:?}", self.dir);
                }
            }
            Command::TurnCw => {
                if self.enable_debug_prints {
                    print!("dir: {:?} -> ", self.dir);
                }
                self.dir = self.dir.turn_cw();
                if self.enable_debug_prints {
                    println!("{:?}", self.dir);
                }
            }
            Command::Mark => {
                if self.enable_debug_prints {
                    println!("mark := {}", self.pos);
                }
                self.mark = self.pos;
            }
            Command::DrawLine => {
                if self.enable_debug_prints {
                    println!("line: {} -> {}", self.pos, self.mark);
                }
                let idx = self.bitmaps.len() - 1;
                self.bitmaps[idx].draw_line(self.pos, self.mark, self.bucket.current_pixel());
            }
            Command::Fill => {
                if self.enable_debug_prints {
                    println!("fill: {}", self.pos);
                }
                let idx = self.bitmaps.len() - 1;
                self.bitmaps[idx].fill(self.pos, self.bucket.current_pixel());
            }
            Command::AddLayer => {
                if self.bitmaps.len() < 10 {
                    crate::png_utils::write_bitmap_as_png_rgba(
                        self.bitmaps.last().unwrap(),
                        std::fs::File::create(format!("./{}.png", self.iteration)).unwrap(),
                    )
                    .unwrap();
                    if self.enable_debug_prints {
                        println!("LAYER+");
                    }
                    self.bitmaps.push(Bitmap::transparent());
                }
            }
            Command::Compose => {
                if self.enable_debug_prints {
                    println!("LAYER COMPOSE");
                }
                if self.bitmaps.len() > 1 {
                    let bitmap = self.bitmaps.pop().unwrap();
                    let idx = self.bitmaps.len() - 1;
                    self.bitmaps[idx].compose_with(bitmap);
                    crate::png_utils::write_bitmap_as_png_rgba(
                        self.bitmaps.last().unwrap(),
                        std::fs::File::create(format!("./{}.png", self.iteration)).unwrap(),
                    )
                    .unwrap();
                }
            }
            Command::Clip => {
                if self.enable_debug_prints {
                    println!("LAYER CLIP");
                }
                if self.bitmaps.len() > 1 {
                    let bitmap = self.bitmaps.pop().unwrap();
                    let idx = self.bitmaps.len() - 1;
                    self.bitmaps[idx].clip_with(bitmap);
                    crate::png_utils::write_bitmap_as_png_rgba(
                        self.bitmaps.last().unwrap(),
                        std::fs::File::create(format!("./{}.png", self.iteration)).unwrap(),
                    )
                    .unwrap();
                }
            }
            Command::Unknown(b) => {
                if self.enable_debug_prints {
                    println!("UNKNOWN: {:?}", b);
                }
            }
        }
        self.iteration += 1;
        self.bitmaps.last().unwrap()
    }

    pub fn draw_debug_overlay(&self, bitmap: &mut Bitmap) {
        let mut pos = self.pos;
        let mut pixel = Pixel { rgb: RED, a: 128 };
        bitmap.set(pos.move_(Direction::Up), pixel);
        bitmap.set(pos.move_(Direction::Right), pixel);
        bitmap.set(pos.move_(Direction::Down), pixel);
        bitmap.set(pos.move_(Direction::Left), pixel);
        pos = self.mark;
        pixel = Pixel { rgb: GREEN, a: 128 };
        bitmap.set(pos.move_(Direction::Up), pixel);
        bitmap.set(pos.move_(Direction::Right), pixel);
        bitmap.set(pos.move_(Direction::Down), pixel);
        bitmap.set(pos.move_(Direction::Left), pixel);
    }
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
