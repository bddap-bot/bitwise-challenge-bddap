// Copied from https://github.com/zesterer/bitwise-examples/blob/b8479bfd485356b3ac399f984e713939ba6a6573/src/lib.rs
// and modified to support input keypress events instead of just is_key_down.
//
// The framebuffer (Output -> write_to) is pure and backend-agnostic. Two
// runners drive a Game: the native one (minifb window) and the wasm one
// (an HTML canvas). minifb has no wasm32 backend, so all minifb usage is
// confined to the `native` feature; the wasm runner lives behind
// `cfg(target_arch = "wasm32")` and shares the same pure rasterizer.

use std::marker::PhantomData;

/// The keys a Game can react to. Owned by this crate (not re-exported from
/// minifb) so the type exists identically on every backend; each runner
/// translates its platform key events into these.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Escape,
}

pub trait Game: Sized + 'static {
    const NAME: &'static str;
    const WIDTH: usize;
    const HEIGHT: usize;

    fn init() -> u64;

    fn tick(prev: u64, input: &Input<'_, Self>, output: &mut Output<'_, Self>) -> u64;

    /// Run the game to completion on the platform's default backend.
    ///
    /// Native: opens a minifb window. Wasm: drives an HTML canvas via
    /// requestAnimationFrame. The wasm path never returns on its own (the rAF
    /// loop keeps running), so we park the thread to satisfy the `-> !` type.
    fn run() -> ! {
        #[cfg(feature = "native")]
        {
            native::run::<Self>()
        }

        #[cfg(all(target_arch = "wasm32", not(feature = "native")))]
        {
            // The rAF loop owns the game from here; start() (the wasm-bindgen
            // entry point) is what actually kicks it off, so run() is unused on
            // wasm. Keep it total for the `-> !` contract.
            loop {
                std::hint::spin_loop();
            }
        }

        #[cfg(not(any(feature = "native", target_arch = "wasm32")))]
        {
            panic!("no backend: build with the `native` feature or for wasm32");
        }
    }
}

pub struct Input<'a, G: Game> {
    keys_pressed: Vec<Key>,
    tick: u64,
    phantom: PhantomData<&'a mut G>,
}

impl<'a, G: Game> Input<'a, G> {
    /// Build an Input for one tick from the keys newly pressed this tick.
    fn new(tick: u64, keys_pressed: Vec<Key>) -> Self {
        Self {
            keys_pressed,
            tick,
            phantom: PhantomData,
        }
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Keys that transitioned to pressed this tick (no auto-repeat), mirroring
    /// minifb's `get_keys_pressed(KeyRepeat::No)`.
    pub fn get_keys_pressed(&self) -> impl Iterator<Item = Key> + '_ {
        self.keys_pressed.iter().copied()
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

    /// Rasterize the collected shapes into a WIDTH*HEIGHT framebuffer. Each
    /// pixel is `u32::from_le_bytes([r, g, b, 255])`, whose little-endian byte
    /// order is exactly canvas ImageData's RGBA layout (no channel swap needed
    /// when blitting to a 2d context).
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

#[cfg(feature = "native")]
mod native {
    use super::{Game, Input, Key, Output};
    use minifb::{KeyRepeat, Window, WindowOptions};
    use std::time::Duration;

    fn from_minifb(key: minifb::Key) -> Option<Key> {
        Some(match key {
            minifb::Key::Up => Key::Up,
            minifb::Key::Down => Key::Down,
            minifb::Key::Left => Key::Left,
            minifb::Key::Right => Key::Right,
            minifb::Key::Escape => Key::Escape,
            _ => return None,
        })
    }

    pub fn run<G: Game>() -> ! {
        let mut win = Window::new(G::NAME, G::WIDTH, G::HEIGHT, WindowOptions::default()).unwrap();

        win.limit_update_rate(Some(Duration::from_micros(16600)));

        let mut state = G::init();

        let mut tick = 0;
        while win.is_open() && !win.is_key_down(minifb::Key::Escape) {
            let mut buf = vec![0; G::WIDTH * G::HEIGHT];

            let keys_pressed = win
                .get_keys_pressed(KeyRepeat::No)
                .into_iter()
                .filter_map(from_minifb)
                .collect();
            let input = Input::<G>::new(tick, keys_pressed);
            let mut output = Output::<G>::new();

            state = G::tick(state, &input, &mut output);

            output.write_to(&mut buf);

            win.update_with_buffer(&buf, G::WIDTH, G::HEIGHT).unwrap();

            tick += 1;
        }

        std::process::exit(0)
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::{Game, Input, Key, Output};
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;

    fn from_event_code(code: &str) -> Option<Key> {
        Some(match code {
            "ArrowUp" | "KeyW" => Key::Up,
            "ArrowDown" | "KeyS" => Key::Down,
            "ArrowLeft" | "KeyA" => Key::Left,
            "ArrowRight" | "KeyD" => Key::Right,
            "Escape" => Key::Escape,
            _ => return None,
        })
    }

    /// Find a `<canvas id="game">` if present, else create one and append it to
    /// the body, then size it to the game's pixel dimensions.
    fn canvas<G: Game>(document: &web_sys::Document) -> web_sys::HtmlCanvasElement {
        let canvas = document
            .get_element_by_id("game")
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .unwrap_or_else(|| {
                let c = document
                    .create_element("canvas")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .unwrap();
                c.set_id("game");
                document.body().unwrap().append_child(&c).unwrap();
                c
            });
        canvas.set_width(G::WIDTH as u32);
        canvas.set_height(G::HEIGHT as u32);
        canvas
    }

    /// Drive `G` on an HTML canvas. Installs a keydown listener that feeds a
    /// shared "pressed this tick" buffer, then runs a requestAnimationFrame
    /// loop that ticks the game at roughly its native cadence and blits the
    /// framebuffer to the canvas via put_image_data.
    pub fn run<G: Game>() {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let canvas = canvas::<G>(&document);
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        // Keys pressed since the last tick consumed them. Shared between the
        // keydown listener and the rAF loop.
        let pressed: Rc<RefCell<Vec<Key>>> = Rc::new(RefCell::new(Vec::new()));

        {
            let pressed = pressed.clone();
            let on_keydown = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
                move |event: web_sys::KeyboardEvent| {
                    if let Some(key) = from_event_code(&event.code()) {
                        // Stop arrow keys from scrolling the page.
                        event.prevent_default();
                        pressed.borrow_mut().push(key);
                    }
                },
            );
            document
                .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
                .unwrap();
            // The listener must outlive this function; leak it (lives for the
            // page's lifetime, which is what we want).
            on_keydown.forget();
        }

        let mut state = G::init();
        let mut framebuffer = vec![0u32; G::WIDTH * G::HEIGHT];
        let mut tick: u64 = 0;
        // The native runner limits to ~60Hz (16600us). rAF already fires at the
        // display refresh (~60Hz on most monitors), so one tick per frame keeps
        // the original cadence without extra timing machinery.

        // Standard wasm-bindgen rAF pattern: a closure that reschedules itself.
        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();
        let pressed_loop = pressed.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            let keys: Vec<Key> = std::mem::take(&mut *pressed_loop.borrow_mut());
            let input = Input::<G>::new(tick, keys);
            let mut output = Output::<G>::new();
            state = G::tick(state, &input, &mut output);

            for px in framebuffer.iter_mut() {
                *px = 0;
            }
            output.write_to(&mut framebuffer);
            blit(&context, &framebuffer, G::WIDTH as u32, G::HEIGHT as u32);

            tick += 1;
            request_animation_frame(f.borrow().as_ref().unwrap());
        }));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }

    /// Reinterpret the u32 framebuffer as RGBA bytes and paint it onto the 2d
    /// context. The framebuffer's little-endian bytes are already [R,G,B,255]
    /// per pixel, matching ImageData's expected RGBA order exactly.
    fn blit(context: &web_sys::CanvasRenderingContext2d, framebuffer: &[u32], w: u32, h: u32) {
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(framebuffer.as_ptr() as *const u8, framebuffer.len() * 4)
        };
        let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(bytes),
            w,
            h,
        )
        .unwrap();
        context.put_image_data(&image, 0.0, 0.0).unwrap();
    }

    fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .expect("requestAnimationFrame failed");
    }
}

/// Entry point for the wasm runner; called by a game's `#[wasm_bindgen(start)]`.
#[cfg(target_arch = "wasm32")]
pub fn run_in_canvas<G: Game>() {
    wasm::run::<G>();
}
