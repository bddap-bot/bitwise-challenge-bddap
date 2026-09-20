use std::marker::PhantomData;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    East = 0,
    North = 1,
    West = 2,
    South = 3,
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => Direction::East,
            1 => Direction::North,
            2 => Direction::West,
            3 => Direction::South,
            _ => unreachable!(),
        }
    }
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
    elapsed: f64,
    directions: Vec<Direction>,
    buffer: Vec<u32>,
    phantom: PhantomData<G>,
}

impl<G: Game> Default for Runner<G> {
    fn default() -> Self {
        Self {
            state: G::init(),
            tick: 0,
            elapsed: 1000.0 / 60.0,
            directions: Vec::new(),
            buffer: vec![0; G::WIDTH * G::HEIGHT],
            phantom: PhantomData,
        }
    }
}

impl<G: Game> Runner<G> {
    pub fn frame(&mut self, elapsed_ms: f64, directions: &[Direction]) -> &[u32] {
        self.directions.extend_from_slice(directions);
        self.elapsed += elapsed_ms.clamp(0.0, 100.0);
        while self.elapsed >= 1000.0 / 60.0 {
            let input = Input {
                directions: &self.directions,
                tick: self.tick,
                phantom: PhantomData,
            };
            let mut output = Output::new();
            self.state = G::tick(self.state, &input, &mut output);
            self.buffer.fill(0);
            output.write_to(&mut self.buffer);
            self.directions.clear();
            self.tick += 1;
            self.elapsed -= 1000.0 / 60.0;
        }
        &self.buffer
    }
}

pub struct Input<'a, G: Game> {
    directions: &'a [Direction],
    tick: u64,
    phantom: PhantomData<G>,
}

impl<G: Game> Input<'_, G> {
    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn directions(&self) -> &[Direction] {
        self.directions
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub use crate::desktop::run;

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
                            if pos[0] > 0
                                && pos[0] < G::WIDTH as i32
                                && pos[1] > 0
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
    use crate::longsnake::Snake;

    #[test]
    fn presentation_rate_does_not_change_simulation() {
        let mut fast = Runner::<Snake>::default();
        let mut slow = Runner::<Snake>::default();
        fast.frame(0.0, &[]);
        slow.frame(0.0, &[]);
        for _ in 0..120 {
            fast.frame(10.0, &[]);
        }
        for _ in 0..30 {
            slow.frame(40.0, &[]);
        }
        assert_eq!(fast.tick, slow.tick);
        assert_eq!(fast.state, slow.state);
        assert_eq!(fast.buffer, slow.buffer);
    }

    #[test]
    fn input_survives_frames_without_a_simulation_step() {
        let mut game = Runner::<Snake>::default();
        game.frame(0.0, &[]);
        let tick = game.tick;
        game.frame(1.0, &[Direction::North]);
        assert_eq!(game.tick, tick);
        assert_eq!(game.directions, [Direction::North]);
        game.frame(20.0, &[]);
        assert!(game.directions.is_empty());
    }

    #[test]
    fn long_pause_has_bounded_catch_up() {
        let mut game = Runner::<Snake>::default();
        game.frame(0.0, &[]);
        let tick = game.tick;
        game.frame(60_000.0, &[]);
        assert!(game.tick - tick <= 6);
    }
}
