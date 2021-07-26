#![allow(clippy::wildcard_imports)]

use game_state::GameState;
use seed::{prelude::*, *};

const STORAGE_KEY: &str = "game_state";
const LEFT_KEY: &str = "ArrowLeft";
const RIGHT_KEY: &str = "ArrowRight";
const UP_KEY: &str = "ArrowUp";
const DOWN_KEY: &str = "ArrowDown";

mod game_state;

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
pub struct Model {
    game_state: game_state::GameState,
}

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.stream(streams::window_event(Ev::KeyDown, |ev| {
        Msg::Move(ev.unchecked_into())
    }));

    Model {
        game_state: LocalStorage::get(STORAGE_KEY).unwrap_or_default(),
    }
}

// ------ ------
//    Update
// ------ ------

// `Msg` describes the different events you can modify state with.
enum Msg {
    Move(web_sys::KeyboardEvent),
    NewGame,
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Move(ev) => {
            ev.prevent_default();

            match ev.key().as_str() {
                LEFT_KEY | "a" => model.game_state.move_tiles(game_state::Direction::Left),
                RIGHT_KEY | "d" => model.game_state.move_tiles(game_state::Direction::Right),
                UP_KEY | "w" => model.game_state.move_tiles(game_state::Direction::Up),
                DOWN_KEY | "s" => model.game_state.move_tiles(game_state::Direction::Down),
                _ => (),
            };
        }
        Msg::NewGame => {
            model.game_state = GameState::default();
        }
    }
    LocalStorage::insert(STORAGE_KEY, &model.game_state).expect("save game state to LocalStorage");
}

// ------ ------
//     View
// ------ ------

// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    div![
        C!["container"],
        view_heading(),
        view_above(),
        div![
            C!["game-container"],
            view_grid(),
            view_tiles(&model.game_state)
        ],
        hr!(),
        view_credits()
    ]
}

fn view_heading() -> Node<Msg> {
    div![C!["heading"], h1![C!["title"], "Seed2048"]]
}

fn view_above() -> Node<Msg> {
    div![
        C!["above-game"],
        p![
            C!["game-intro"],
            "Built with ",
            strong!("Seed"),
            ", a Rust framework."
        ],
        a![
            C!["restart-button"],
            "New Game",
            ev(Ev::Click, |_| Msg::NewGame)
        ]
    ]
}

fn view_grid() -> Node<Msg> {
    let mut cells = Vec::new();
    for _ in 0..4 {
        cells.push(div![C!["grid-cell"]]);
    }

    let mut rows = Vec::new();
    for _ in 0..4 {
        rows.push(div![C!["grid-row"], &cells]);
    }

    div![C!["grid-container"], &rows]
}

fn tile_name(index: usize, tile: game_state::Tile) -> String {
    let state = tile.get_state();
    let value = tile.get_value();

    format!(
        "tile tile-{} tile-position-{}-{}{}",
        if value <= 2048 {
            value.to_string()
        } else {
            "super".to_string()
        },
        index % 4 + 1,
        index / 4 + 1,
        state
    )
}

fn view_tile(index: usize, tile: game_state::Tile) -> Node<Msg> {
    let value = tile.get_value();
    let name = tile_name(index, tile);

    if let Some(prev) = tile.get_prev() {
        let prev_name = tile_name(prev, tile);
        div![C![name], div![C!["tile-inner"], value]]
    } else {
        div![C![name], div![C!["tile-inner"], value]]
    }
}

fn view_tiles(game_state: &game_state::GameState) -> Node<Msg> {
    let mut tiles = Vec::new();
    for (i, tile) in game_state.get_tiles() {
        tiles.push(view_tile(i, tile));
    }

    div![C!["tile-container"], tiles]
}

fn view_credits() -> Node<Msg> {
    p![
        "Created by ",
        strong!("Byeong Min Park."),
        " Based on ",
        a![
            "2048 by Gabriele Cirulli.",
            attrs! {At::Href => "https://play2048.co/"}
        ]
    ]
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
