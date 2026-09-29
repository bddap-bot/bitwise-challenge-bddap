// Copied from https://github.com/zesterer/bitwise-examples/blob/b8479bfd485356b3ac399f984e713939ba6a6573/src/lib.rs

use std::marker::PhantomData;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    East = 0,
    North = 1,
    West = 2,
    South = 3,
}

pub trait Game: Sized + 'static {
    const NAME: &'static str;
    const WIDTH: usize;
    const HEIGHT: usize;

    fn init() -> u64;

    fn tick(prev: u64, input: &Input<'_, Self>, output: &mut Output<'_, Self>) -> u64;
}

pub struct Runner<G: Game> {
    state: u64,
    tick: u64,
    directions: Vec<Direction>,
    buf: Vec<u32>,
    phantom: PhantomData<G>,
}

impl<G: Game> Default for Runner<G> {
    fn default() -> Self {
        Self {
            state: G::init(),
            tick: 0,
            directions: Vec::new(),
            buf: vec![0; G::WIDTH * G::HEIGHT],
            phantom: PhantomData,
        }
    }
}

impl<G: Game> Runner<G> {
    pub fn push(&mut self, direction: Direction) {
        self.directions.push(direction);
    }

    pub fn step(&mut self) -> &[u32] {
        let input = Input {
            directions: &self.directions,
            tick: self.tick,
            phantom: PhantomData,
        };
        let mut output = Output::new();

        self.state = G::tick(self.state, &input, &mut output);

        self.buf.fill(0);
        output.write_to(&mut self.buf);

        self.directions.clear();
        self.tick += 1;
        &self.buf
    }
}

pub struct Input<'a, G: Game> {
    directions: &'a [Direction],
    tick: u64,
    phantom: PhantomData<&'static mut G>,
}

impl<'a, G: Game> Input<'a, G> {
    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn directions(&self) -> &[Direction] {
        self.directions
    }
}

pub struct Output<'a, G: Game> {
    shapes: Vec<Shape>,
    phantom: PhantomData<&'a mut G>,
}

enum Shape {
    Rect {
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        color: [u8; 3],
    },
}

impl<'a, G: Game> Output<'a, G> {
    pub fn rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: [u8; 3]) {
        self.shapes.push(Shape::Rect { x, y, w, h, color });
    }

    fn new() -> Self {
        Self {
            shapes: Vec::new(),
            phantom: PhantomData,
        }
    }

    fn write_to(self, buf: &mut [u32]) {
        for shape in self.shapes {
            match shape {
                Shape::Rect { x, y, w, h, color } => {
                    for j in 0..h {
                        for i in 0..w {
                            let pos = [x + i as i32, y + j as i32];
                            if pos[0] >= 0
                                && pos[0] < G::WIDTH as i32
                                && pos[1] >= 0
                                && pos[1] < G::HEIGHT as i32
                            {
                                buf[pos[1] as usize * G::WIDTH + pos[0] as usize] =
                                    u32::from_le_bytes([color[0], color[1], color[2], 255]);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Probe;

    impl Game for Probe {
        const NAME: &'static str = "Probe";
        const WIDTH: usize = 2;
        const HEIGHT: usize = 2;

        fn init() -> u64 {
            0
        }

        fn tick(prev: u64, input: &Input<'_, Self>, output: &mut Output<'_, Self>) -> u64 {
            if input.tick() == 0 {
                output.rect(-1, -1, 4, 4, [1, 2, 3]);
            }
            prev + input.directions().len() as u64
        }
    }

    #[test]
    fn each_pushed_direction_reaches_one_tick() {
        let mut game = Runner::<Probe>::default();
        game.push(Direction::North);
        game.push(Direction::East);
        game.step();
        game.step();
        game.push(Direction::West);
        game.step();
        assert_eq!(game.state, 3);
    }

    #[test]
    fn rects_clip_to_the_frame_and_frames_start_blank() {
        let mut game = Runner::<Probe>::default();
        assert_eq!(game.step(), [u32::from_le_bytes([1, 2, 3, 255]); 4]);
        assert_eq!(game.step(), [0; 4]);
    }
}
