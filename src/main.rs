use std::{collections::HashMap, time::Duration};

use bitwise_challenge_bddap::{
    game::{Direction, Game, Runner},
    longsnake::Snake,
};
use gilrs::{Axis, Button, EventType, Gilrs};
use minifb::{Key, KeyRepeat, Window, WindowOptions};

fn main() {
    let mut win = Window::new(
        Snake::NAME,
        Snake::WIDTH,
        Snake::HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    win.limit_update_rate(Some(Duration::from_micros(16600)));

    let mut game = Runner::<Snake>::default();
    let mut gilrs = Gilrs::new().ok();
    let mut sticks = HashMap::new();

    while win.is_open() && !win.is_key_down(Key::Escape) {
        for key in win.get_keys_pressed(KeyRepeat::No) {
            match key {
                Key::Right | Key::D => game.push(Direction::East),
                Key::Up | Key::W => game.push(Direction::North),
                Key::Left | Key::A => game.push(Direction::West),
                Key::Down | Key::S => game.push(Direction::South),
                _ => {}
            }
        }

        if let Some(gilrs) = &mut gilrs {
            while let Some(event) = gilrs.next_event() {
                match event.event {
                    EventType::ButtonPressed(Button::DPadRight, _) => game.push(Direction::East),
                    EventType::ButtonPressed(Button::DPadUp, _) => game.push(Direction::North),
                    EventType::ButtonPressed(Button::DPadLeft, _) => game.push(Direction::West),
                    EventType::ButtonPressed(Button::DPadDown, _) => game.push(Direction::South),
                    EventType::Disconnected => {
                        sticks.remove(&event.id);
                    }
                    _ => {}
                }
            }

            for (id, pad) in gilrs.gamepads() {
                let held = sticks.entry(id).or_insert(None);
                let direction = stick_direction(
                    pad.value(Axis::LeftStickX),
                    pad.value(Axis::LeftStickY),
                    *held,
                );
                if direction != *held {
                    *held = direction;
                    if let Some(direction) = direction {
                        game.push(direction);
                    }
                }
            }
        }

        win.update_with_buffer(game.step(), Snake::WIDTH, Snake::HEIGHT)
            .unwrap();
    }
}

fn stick_direction(x: f32, y: f32, held: Option<Direction>) -> Option<Direction> {
    if let Some(direction) = held {
        let along = match direction {
            Direction::East => x,
            Direction::North => y,
            Direction::West => -x,
            Direction::South => -y,
        };
        if along > 0.35 && along + 0.15 >= x.abs().max(y.abs()) {
            return held;
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

#[cfg(test)]
mod tests {
    use super::*;
    use Direction::*;

    #[test]
    fn stick_deadzone_and_dominant_axis() {
        for (x, y, expected) in [
            (0.45, 0.0, None),
            (0.4, -0.4, None),
            (0.6, 0.0, Some(East)),
            (0.8, 0.6, Some(East)),
            (-0.8, 0.6, Some(West)),
            (0.6, 0.8, Some(North)),
            (0.6, -0.8, Some(South)),
        ] {
            assert_eq!(stick_direction(x, y, None), expected, "({x}, {y})");
        }
    }

    #[test]
    fn stick_hysteresis_and_release() {
        for (x, y, held, expected) in [
            (0.70, 0.75, East, Some(East)),
            (0.75, 0.70, North, Some(North)),
            (-0.70, -0.75, West, Some(West)),
            (0.75, -0.70, South, Some(South)),
            (0.40, 0.0, East, Some(East)),
            (0.20, 0.0, East, None),
            (0.50, 0.90, East, Some(North)),
            (0.0, 0.0, East, None),
        ] {
            assert_eq!(
                stick_direction(x, y, Some(held)),
                expected,
                "({x}, {y}) holding {held:?}"
            );
        }
    }
}
