use rand::prelude::IteratorRandom;
use rand::thread_rng;
use rand::Rng;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, Deserialize, Serialize)]
pub struct Tile {
    value: usize,
    state: TileState,
    prev_pos: Option<usize>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
enum TileState {
    New,
    Static,
    Merged,
}

impl Tile {
    fn new(value: usize) -> Tile {
        Tile {
            value,
            state: TileState::New,
            prev_pos: None,
        }
    }

    fn update(&mut self, value: usize, state: TileState) {
        self.value = value;
        self.state = state;
    }

    pub fn get_value(&self) -> usize {
        self.value
    }

    pub fn get_prev(&self) -> Option<usize> {
        self.prev_pos
    }

    pub fn get_state(&self) -> String {
        match self.state {
            TileState::New => String::from(" tile-new"),
            TileState::Merged => String::from(" tile-merged"),
            TileState::Static => String::new(),
        }
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Tile) -> bool {
        self.value == other.value
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn increment(self) -> (i32, i32, i32) {
        match self {
            Direction::Left => (0, 1, 0),
            Direction::Right => (15, -1, 0),
            Direction::Up => (0, 4, 1),
            Direction::Down => (15, -4, -1),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameState {
    grid: [Option<Tile>; 16],
    score: usize,
    over: bool,
    won: bool,
    generate_tiles: bool,
}

impl GameState {
    fn new(grid: [Option<Tile>; 16], generate_tiles: bool) -> GameState {
        GameState {
            grid,
            score: 0,
            over: false,
            won: false,
            generate_tiles: generate_tiles,
        }
    }

    fn is_game_over(&mut self) -> bool {
        self.over || self.won
    }

    pub fn add_random_tile(&mut self) {
        if !self.generate_tiles {
            return;
        }

        let mut rng = thread_rng();

        let grid_empty = self.grid.iter_mut().filter(|tile| tile.is_none());

        if let Some(empty) = grid_empty.choose(&mut rng) {
            let value = match rng.gen::<f64>() {
                x if x > 0.9 => 4,
                _ => 2,
            };

            *empty = Some(Tile::new(value));
        }
    }

    fn prepare_move(&mut self) {
        for i in 0..16 {
            self.grid
                .get_mut(i)
                .and_then(|tile| tile.as_mut())
                .map(|tile| {
                    tile.state = TileState::New;
                    tile.prev_pos = Some(i);
                });
        }
    }

    pub fn move_tiles(&mut self, direction: Direction) {
        if self.is_game_over() {
            return;
        }

        self.prepare_move();

        let mut moved = false;
        let mut index = direction.increment().0;

        for _ in 0..4 {
            let mut next = index;

            for _ in 0..4 {
                if let Some(mut curr_tile) = self.grid[index as usize] {
                    let mut moved_tile = false;
                    let prev = next - direction.increment().1;

                    if prev >= 0 && prev < 16 {
                        if let Some(mut merge_tile) = self.grid[prev as usize] {
                            if merge_tile.state != TileState::Merged && merge_tile == curr_tile {
                                merge_tile.update(merge_tile.value * 2, TileState::Merged);

                                self.grid[prev as usize] = Some(merge_tile);
                                self.grid[index as usize] = None;
                                moved_tile = true;

                                self.score += merge_tile.value;
                                if merge_tile.value == 2048 {
                                    self.won = true;
                                }
                            }
                        }
                    }

                    if !moved_tile {
                        if index == next {
                            curr_tile.update(curr_tile.value, TileState::Static);
                            self.grid[index as usize] = Some(curr_tile);

                            next += direction.increment().1;
                        } else {
                            curr_tile.update(curr_tile.value, TileState::Static);

                            self.grid[next as usize] = Some(curr_tile);
                            self.grid[index as usize] = None;
                            moved_tile = true;

                            next += direction.increment().1;
                        }
                    }

                    moved |= moved_tile;
                }

                index += direction.increment().1;
            }

            index = (index + direction.increment().2 + 16) % 16;
        }

        if moved {
            self.add_random_tile();
        }
    }

    pub fn get_tiles(&self) -> impl Iterator<Item = (usize, Tile)> + '_ {
        self.grid
            .iter()
            .enumerate()
            .filter_map(|(i, t)| match t {
                None => None,
                Some(tile) => Some((i, *tile)),
            })
            .flat_map(|(i, tile)| match tile.state {
                TileState::Merged => vec![
                    (
                        i,
                        Tile {
                            value: tile.value / 2,
                            state: TileState::Static,
                            prev_pos: tile.prev_pos,
                        },
                    ),
                    (i, tile),
                ],
                _ => vec![(i, tile)],
            })
    }
}

impl Default for GameState {
    fn default() -> Self {
        let mut game_state = GameState::new([None; 16], true);
        for _ in 0..2 {
            game_state.add_random_tile();
        }
        game_state
    }
}

impl PartialEq for GameState {
    fn eq(&self, other: &GameState) -> bool {
        self.grid == other.grid
    }
}

#[cfg(test)]
mod tests {
    use crate::game_state::{Direction, GameState, Tile};

    fn to_grid(from: [usize; 16]) -> [Option<Tile>; 16] {
        let mut to = [None; 16];
        for i in 0..from.len() {
            if from[i] != 0 {
                to[i].insert(Tile::new(from[i]));
            }
        }
        to
    }

    fn from_grid(from: [Option<Tile>; 16]) -> [usize; 16] {
        let mut to = [0; 16];
        for i in 0..from.len() {
            if let Some(tile) = from[i] {
                to[i] = tile.value;
            }
        }
        to
    }

    #[test]
    fn test_basic() {
        struct TestCase<'a> {
            name: &'a str,
            curr: [usize; 16],
            want: [usize; 16],
            moves: Vec<Direction>,
        }

        let tests = [
            TestCase {
                name: "Basic: Left",
                curr: [2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2],
                want: [2, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0],
                moves: vec![Direction::Left],
            },
            TestCase {
                name: "Basic: Right",
                curr: [2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2],
                want: [0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 2],
                moves: vec![Direction::Right],
            },
            TestCase {
                name: "Basic: Up",
                curr: [2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2],
                want: [2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                moves: vec![Direction::Up],
            },
            TestCase {
                name: "Basic: Down",
                curr: [2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2],
                want: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2],
                moves: vec![Direction::Down],
            },
        ];

        for t in tests {
            let curr = to_grid(t.curr);
            let mut gs = GameState::new(curr, false);

            for d in &t.moves {
                gs.move_tiles(*d);
            }

            assert_eq!(t.want, from_grid(gs.grid), "{}", t.name);
        }
    }

    #[test]
    fn test_merge() {
        struct TestCase<'a> {
            name: &'a str,
            curr: [usize; 16],
            want: [usize; 16],
            moves: Vec<Direction>,
        }

        let tests = [
            TestCase {
                name: "Merge: Basic Merge",
                curr: [2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                want: [4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                moves: vec![Direction::Left],
            },
            TestCase {
                name: "Merge: Not Merge",
                curr: [2, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0],
                want: [2, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                moves: vec![Direction::Up],
            },
            TestCase {
                name: "Merge: Merge Twice",
                curr: [2, 2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                want: [0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                moves: vec![Direction::Left, Direction::Right],
            },
            TestCase {
                name: "Merge: Merge Twos",
                curr: [2, 2, 2, 2, 0, 2, 2, 2, 0, 0, 2, 2, 0, 0, 0, 2],
                want: [4, 4, 0, 0, 4, 2, 0, 0, 4, 0, 0, 0, 2, 0, 0, 0],
                moves: vec![Direction::Left],
            },
        ];

        for t in tests {
            let curr = to_grid(t.curr);
            let mut gs = GameState::new(curr, false);

            for d in &t.moves {
                gs.move_tiles(*d);
            }

            assert_eq!(t.want, from_grid(gs.grid), "{}", t.name);
        }
    }

    #[test]
    fn test_random_tiles() {
        struct TestCase<'a> {
            name: &'a str,
            curr: [usize; 16],
            want: usize,
            moves: Vec<Direction>,
        }

        let tests = [
            TestCase {
                name: "Random Tile: Valid Move",
                curr: [0, 0, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0],
                want: 3,
                moves: vec![Direction::Left],
            },
            TestCase {
                name: "Random Tile: Invalid Move",
                curr: [0, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0],
                want: 2,
                moves: vec![Direction::Left],
            },
        ];

        for t in tests {
            let curr = to_grid(t.curr);
            let mut gs = GameState::new(curr, true);

            for d in &t.moves {
                gs.move_tiles(*d);
            }

            assert_eq!(
                t.want,
                gs.grid.iter().filter(|x| x.is_some()).count(),
                "{}",
                t.name
            );
        }
    }
}
