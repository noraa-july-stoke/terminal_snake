#![allow(dead_code)]
use rand::Rng;
use std::io;
use std::io::Write;
use std::thread;
use std::time::Duration;
use termion;
use termion::async_stdin;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Clone, Copy)] // Added Clone and Copy here

pub struct Position {
    pub x: u16,
    pub y: u16,
}

pub struct Snake {
    pub direction: Direction,
    pub body: Vec<Position>,
    pub food: Position,
    pub blocks: Vec<Position>,
    pub has_eaten: bool,
}

impl Snake {
    pub fn new() -> Snake {
        let mut rng = rand::thread_rng();
        let (max_x, max_y) = termion::terminal_size().unwrap();
        let food = Position {
            x: rng.gen_range(1..=max_x),
            y: rng.gen_range(1..=max_y),
        };

        Snake {
            direction: Direction::Right,
            body: vec![Position {
                x: max_x / 2,
                y: max_y / 2,
            }],
            food,
            blocks: vec![],
            has_eaten: false,
        }
    }

    pub fn step(&mut self) -> bool {
        let (max_x, max_y) = termion::terminal_size().unwrap();
        let head = self.body.last().unwrap().clone();
        let mut rng = rand::thread_rng();
        let new_head = match self.direction {
            Direction::Up => Position {
                x: head.x,
                y: (head.y + max_y - 1) % max_y,
            },
            Direction::Down => Position {
                x: head.x,
                y: (head.y + 1) % max_y,
            },
            Direction::Left => Position {
                x: (head.x + max_x - 1) % max_x,
                y: head.y,
            },
            Direction::Right => Position {
                x: (head.x + 1) % max_x,
                y: head.y,
            },
        };

        if self.body.contains(&new_head) || self.blocks.contains(&new_head) {
            return false; // Snake has collided with itself or with a block
        }


        if new_head == self.food {
            self.has_eaten = true;
            let mut new_food = Position {
                x: rng.gen_range(0..max_x),
                y: rng.gen_range(0..max_y),
            };
            while self.body.contains(&new_food) {
                new_food = Position {
                    x: rng.gen_range(0..max_x),
                    y: rng.gen_range(0..max_y),
                };
            }
            // spawn a block
            let new_block;
            let mut rng = rand::thread_rng();
            let direction = rng.gen_range(0..4); // choose a random direction for block placement
            let chance: f32 = rng.gen(); // Generate a float between 0 and 1
            if chance < 0.75 && !self.blocks.is_empty() {
                // 75% chance to attach to an existing block to form a "wall"
                let last_block = self.blocks.last().unwrap().clone();
                new_block = match direction {
                    0 => Position {
                        // up
                        x: last_block.x,
                        y: (last_block.y + max_y - 1) % max_y,
                    },
                    1 => Position {
                        // down
                        x: last_block.x,
                        y: (last_block.y + 1) % max_y,
                    },
                    2 => Position {
                        // left
                        x: (last_block.x + max_x - 1) % max_x,
                        y: last_block.y,
                    },
                    _ => Position {
                        // right
                        x: (last_block.x + 1) % max_x,
                        y: last_block.y,
                    },
                };
            } else {
                // 25% chance to start a new "wall"
                new_block = Position {
                    x: rng.gen_range(0..max_x),
                    y: rng.gen_range(0..max_y),
                };
            }

            self.blocks.push(new_block);

            self.food = new_food;
        }

        if !self.has_eaten {
            self.body.remove(0);
        } else {
            self.has_eaten = false;
        }

        self.body.push(new_head);
        return true;
    }
    // we are taking a reference to new_head here because we want to
    // modify it in the loop below and we can't do that if we own it
    // (i.e., if we pass it by value). We need to pass it by reference
    // so that we can modify it in place. We cant modify owned values
    // in place because Rust would then have to drop the old value and
    // move the new value into its place, which is not possible because
    // we are borrowing it immutably in the loop below.
}

pub fn main() {
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut snake = Snake::new();

    let stdin = async_stdin();
    let mut keys = stdin.keys();

    loop {
        stdout.flush().unwrap();
        thread::sleep(Duration::from_millis(50));

        if let Some(Ok(key)) = keys.next() {
            match key {
                Key::Char('w') => snake.direction = Direction::Up,
                Key::Char('s') => snake.direction = Direction::Down,
                Key::Char('a') => snake.direction = Direction::Left,
                Key::Char('d') => snake.direction = Direction::Right,
                _ => {}
            }
        }
        if !snake.step() {
            break; // Snake has collided with itself, end the game
        }

        // snake.step();

        write!(stdout, "{}", termion::clear::All).unwrap();
        for pos in &snake.body {
            write!(stdout, "{}{}", cursor::Goto(pos.x, pos.y), "#").unwrap();
        }

        for pos in &snake.blocks {
            write!(stdout, "{}{}", cursor::Goto(pos.x, pos.y), "#").unwrap();
        }
        write!(
            stdout,
            "{}{}",
            cursor::Goto(snake.food.x, snake.food.y),
            "*"
        )
        .unwrap();
    }
}
