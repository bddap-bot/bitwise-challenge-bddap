use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use gilrs::{Axis, Button, EventType, Gilrs};
use minifb::{Key, KeyRepeat, Window, WindowOptions};

use crate::game::{Direction, Game, Runner};

fn stick_direction(x: f32, y: f32, previous: Option<Direction>) -> Option<Direction> {
    if let Some(direction) = previous {
        let held = match direction {
            Direction::East => x,
            Direction::West => -x,
            Direction::North => y,
            Direction::South => -y,
        };
        if held > 0.35 && held + 0.15 >= x.abs().max(y.abs()) {
            return previous;
        }
    }
    if x.abs().max(y.abs()) < 0.5 {
        None
    } else if x.abs() > y.abs() {
        Some(if x > 0.0 {
            Direction::East
        } else {
            Direction::West
        })
    } else {
        Some(if y > 0.0 {
            Direction::North
        } else {
            Direction::South
        })
    }
}

pub fn run<G: Game>() {
    let mut window = Window::new(G::NAME, G::WIDTH, G::HEIGHT, WindowOptions::default()).unwrap();
    window.limit_update_rate(Some(Duration::from_secs_f64(1.0 / 60.0)));
    let mut game = Runner::<G>::default();
    let mut controllers = Gilrs::new().ok();
    let mut sticks = HashMap::new();
    let mut previous = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut directions = Vec::new();
        for key in window.get_keys_pressed(KeyRepeat::No) {
            if let Some(direction) = match key {
                Key::Right | Key::D => Some(Direction::East),
                Key::Up | Key::W => Some(Direction::North),
                Key::Left | Key::A => Some(Direction::West),
                Key::Down | Key::S => Some(Direction::South),
                _ => None,
            } {
                directions.push(direction);
            }
        }
        if let Some(controllers) = &mut controllers {
            while let Some(event) = controllers.next_event() {
                match event.event {
                    EventType::ButtonPressed(button, _) => {
                        if let Some(direction) = match button {
                            Button::DPadRight => Some(Direction::East),
                            Button::DPadUp => Some(Direction::North),
                            Button::DPadLeft => Some(Direction::West),
                            Button::DPadDown => Some(Direction::South),
                            _ => None,
                        } {
                            directions.push(direction);
                        }
                    }
                    EventType::Disconnected => {
                        sticks.remove(&event.id);
                    }
                    _ => {}
                }
            }
            for (id, pad) in controllers.gamepads() {
                let previous = sticks.entry(id).or_insert(None);
                let direction = stick_direction(
                    pad.value(Axis::LeftStickX),
                    pad.value(Axis::LeftStickY),
                    *previous,
                );
                if *previous != direction {
                    directions.extend(direction);
                    *previous = direction;
                }
            }
        }
        let now = Instant::now();
        let elapsed = now.duration_since(previous).as_secs_f64() * 1000.0;
        previous = now;
        window
            .update_with_buffer(game.frame(elapsed, &directions), G::WIDTH, G::HEIGHT)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stick_hysteresis_and_release() {
        let east = Some(Direction::East);
        assert_eq!(stick_direction(0.70, 0.75, east), east);
        assert_eq!(stick_direction(0.40, 0.0, east), east);
        assert_eq!(stick_direction(0.20, 0.0, east), None);
        assert_eq!(stick_direction(0.50, 0.90, east), Some(Direction::North));
        assert_eq!(stick_direction(0.0, 0.0, east), None);
    }

    #[test]
    fn stick_deadzone_and_dominant_axis() {
        assert_eq!(stick_direction(0.4, -0.4, None), None);
        assert_eq!(stick_direction(0.8, 0.6, None), Some(Direction::East));
        assert_eq!(stick_direction(-0.8, 0.6, None), Some(Direction::West));
        assert_eq!(stick_direction(0.6, 0.8, None), Some(Direction::North));
        assert_eq!(stick_direction(0.6, -0.8, None), Some(Direction::South));
    }
}
