use std::{collections::HashMap, vec};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout},
    prelude::CrosstermBackend,
    style::Stylize,
    widgets::Paragraph,
    Terminal::{self},
};

use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::*;
use std::fs::File;
use std::io::BufReader;

#[derive(Clone)]
struct GameState {
    grid: Vec<Vec<char>>,
    player_position: (i32, i32),
    level: Option<Level>,
    scores: HashMap<Level, (i32, i32)>,
    moves: Vec<MoveDirection>,
}

#[derive(PartialEq, Debug, Clone)]
enum MoveDirection {
    Up,
    Right,
    Down,
    Left,
}
#[derive(PartialEq, Clone, Copy, Eq, Hash, Debug)]
enum Level {
    One,
    Two,
    Three,
    Four,
    Five,
}
#[derive(PartialEq)]
enum Command {
    Quit,
    Move(MoveDirection),
    LevelChoose,
    LevelSelect(Level),
    Reset,
    ReverseMove,
}

enum SoundType {
    Oof,
    BarrelMove,
    BarrelCorrect,
    WinGame,
    BarrelOof,
    PlayerMove,
}

fn main() -> std::io::Result<()> {
    let (mut game_state, mut terminal) = startup();
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    loop {
        if let Event::Key(key) = event::read()? {
            let ret = do_action(&mut game_state, key, &sink);
            if ret == 1 {
                break;
            }
            finish_if_solved(&mut game_state);

            let _ = terminal.draw(|frame| {
                let areas = Layout::vertical(vec![Constraint::Length(1); game_state.grid.len()])
                    .split(frame.area());

                // use the simpler short-hand syntax
                game_state.grid.iter().enumerate().for_each(|(idx, row)| {
                    frame.render_widget(Paragraph::new(String::from_iter(row)).blue(), areas[idx]);
                });
            });
        }
    }
    ratatui::restore();
    Ok(())
}

fn do_action(game_state: &mut GameState, key: KeyEvent, sink: &Sink) -> i32 {
    if let Some(command) = read_input(key) {
        return match command {
            Command::Quit => 1,
            Command::Reset => {
                if let Some(cur_level) = game_state.level {
                    if game_state.level.is_some() {
                        start_level(game_state, cur_level);
                    }
                }
                return 0;
            }
            Command::LevelChoose => {
                choose_level(game_state);
                return 0;
            }
            Command::Move(direction) => {
                player_move(direction, game_state, true, sink);
                return 0;
            }
            Command::LevelSelect(level) => {
                start_level(game_state, level);
                game_state.level = Some(level);
                return 0;
            }
            Command::ReverseMove => {
                if game_state.moves.len() == 0 {
                    return 0;
                }
                let direction = match game_state.moves.pop().unwrap() {
                    MoveDirection::Up => MoveDirection::Down,
                    MoveDirection::Down => MoveDirection::Up,
                    MoveDirection::Left => MoveDirection::Right,
                    MoveDirection::Right => MoveDirection::Left,
                };
                player_move(direction, game_state, false, sink);
                return 0;
            }
        };
    }
    return -1;
}

fn startup() -> (GameState, Terminal<CrosstermBackend<std::io::Stdout>>) {
    let mut terminal: Terminal<CrosstermBackend<std::io::Stdout>> = ratatui::init();
    let game_state = GameState {
        grid: vec!["Welcome! Press \"m\" to go to level select."
            .chars()
            .collect::<Vec<_>>()],
        player_position: (0, 0),
        level: None,
        scores: HashMap::new(),
        moves: vec![],
    };
    let _ = terminal.draw(|frame| {
        let areas = Layout::vertical(vec![Constraint::Length(1); game_state.grid.len()])
            .split(frame.area());

        // use the simpler short-hand syntax
        game_state.grid.iter().enumerate().for_each(|(idx, row)| {
            frame.render_widget(Paragraph::new(String::from_iter(row)).blue(), areas[idx]);
        });
    });
    return (game_state, terminal);
}

fn finish_if_solved(game_state: &mut GameState) {
    if game_state
        .grid
        .iter()
        .flatten()
        .find(|c| **c == '$')
        .is_none()
        && game_state.level.is_some()
    {
        let cur_level = game_state.level.unwrap();
        let (high_score, cur_score) = game_state.scores.get(&cur_level).unwrap();
        if cur_score < high_score || *high_score == 0 {
            game_state.grid = vec![format!("You won! New record - you completed this level in {} moves. Your lowest number of moves for this level previously was {}. Press \"m\" to go back to the main menu.", cur_score, high_score)
            .chars()
            .collect::<Vec<_>>()];
            game_state.scores.insert(cur_level, (*cur_score, 0));
        } else {
            game_state.grid = vec![format!("You won! You completed this level in {} moves. Your lowest number of moves for this level is {}. Press \"m\" to go back to the main menu.", cur_score, high_score)
                .chars()
                .collect::<Vec<_>>()];
        }
        game_state.level = None;
    }
}
fn choose_level(game_state: &mut GameState) {
    game_state.grid = vec![
        "Choose level:".chars().collect::<Vec<_>>(),
        "1 - Tutorial".chars().collect::<Vec<_>>(),
        "2 - Easy".chars().collect::<Vec<_>>(),
        "3 - Medium".chars().collect::<Vec<_>>(),
        "4 - Hard".chars().collect::<Vec<_>>(),
    ];
}

fn start_level(game_state: &mut GameState, level: Level) {
    game_state.moves = vec![];
    game_state
        .scores
        .entry(level)
        .and_modify(|val| val.1 = 0)
        .or_insert((0, 0));

    (game_state.grid, game_state.player_position) = match level {
        Level::One => (
            vec![
                vec!['#', '#', '#', '#', '#'],
                vec!['#', ' ', ' ', ' ', '#'],
                vec!['#', '.', '$', '@', '#'],
                vec!['#', ' ', ' ', ' ', '#'],
                vec!['#', '#', '#', '#', '#'],
            ],
            (3, 2),
        ),
        Level::Two => (
            vec![
                vec![' ', ' ', ' ', ' ', ' ', '#', '#', '#', '#'],
                vec!['#', '#', '#', '#', '#', '#', ' ', ' ', '#'],
                vec!['#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#'],
                vec!['#', ' ', ' ', ' ', ' ', ' ', ' ', '.', '#'],
                vec!['#', '@', ' ', '#', '#', '#', '#', '#', '#', '#'],
                vec!['#', '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#'],
                vec![' ', '#', ' ', '#', ' ', '#', ' ', ' ', ' ', '#'],
                vec![' ', '#', ' ', ' ', ' ', ' ', ' ', '$', ' ', '#'],
                vec![' ', '#', ' ', ' ', ' ', '#', '#', '#', '#', '#'],
                vec![' ', '#', '#', '#', '#', '#'],
            ],
            (1, 4),
        ),
        Level::Three => (
            vec![
                vec![
                    '#', '#', '#', '#', '#', ' ', ' ', '#', '#', '#', '#', ' ', ' ', '#', '#', '#',
                    '#', '#',
                ],
                vec![
                    '#', ' ', ' ', ' ', '#', '#', '#', '#', ' ', ' ', '#', '#', '#', '#', ' ', ' ',
                    ' ', '#',
                ],
                vec![
                    '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
                    ' ', '#',
                ],
                vec![
                    '#', '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#', '#', '#', ' ', ' ', ' ',
                    '#', '#',
                ],
                vec![
                    ' ', '#', '#', ' ', '$', ' ', ' ', '#', ' ', '.', '.', ' ', '$', ' ', '@', '#',
                    '#',
                ],
                vec![
                    '#', '#', ' ', ' ', '#', '#', ' ', ' ', ' ', '#', '#', '#', '#', ' ', ' ', ' ',
                    '#', '#',
                ],
                vec![
                    '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
                    ' ', '#',
                ],
                vec![
                    '#', ' ', ' ', ' ', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', ' ', ' ',
                    ' ', '#',
                ],
                vec![
                    '#', '#', '#', '#', '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#', '#', '#',
                    '#', '#',
                ],
            ],
            (3, 2),
        ),
        Level::Four => (
            vec![
                vec![' ', '#', '#', '#', '#', '#'],
                vec!['#', '#', ' ', ' ', ' ', '#'],
                vec!['#', ' ', ' ', ' ', ' ', '#', '#'],
                vec!['#', ' ', ' ', '#', ' ', ' ', '#'],
                vec!['#', ' ', '$', '#', ' ', '.', '#', '#', '#'],
                vec!['#', ' ', ' ', '#', '*', '.', ' ', ' ', '#'],
                vec!['#', ' ', '$', ' ', '$', '.', ' ', ' ', '#'],
                vec!['#', ' ', ' ', '#', '$', '.', '#', '#', '#'],
                vec!['#', '#', '#', '#', ' ', '.', '#'],
                vec![' ', ' ', '#', '#', '$', '.', '#'],
                vec![' ', ' ', '#', ' ', '$', '*', '#'],
                vec![' ', ' ', '#', ' ', ' ', '@', '#'],
                vec![' ', ' ', '#', '#', '#', '#', '#'],
            ],
            (3, 2),
        ),
        Level::Five => (
            vec![
                vec![' ', '#', '#', '#', '#'],
                vec!['#', '#', ' ', ' ', '#', '#', '#'],
                vec!['#', ' ', ' ', ' ', ' ', ' ', '#', '#', '#'],
                vec!['#', ' ', '#', '*', '*', '*', '.', ' ', '#'],
                vec!['#', ' ', ' ', '*', ' ', ' ', '#', ' ', '#'],
                vec!['#', ' ', ' ', '*', ' ', ' ', ' ', ' ', '#'],
                vec!['#', ' ', ' ', '*', '*', '*', '#', '#', '#', '#'],
                vec!['#', '#', '#', '#', ' ', ' ', '*', ' ', ' ', '#'],
                vec![' ', '#', ' ', '*', ' ', ' ', '*', ' ', ' ', '#'],
                vec![' ', '#', ' ', '$', '*', '*', ' ', ' ', ' ', '#'],
                vec![' ', '#', ' ', ' ', ' ', '@', '#', ' ', ' ', '#'],
                vec![' ', '#', '#', '#', '#', '#', '#', '#', '#', '#'],
            ],
            (3, 2),
        ),
    };
}

fn player_move(
    direction: MoveDirection,
    game_state: &mut GameState,
    record_as_move: bool,
    sink: &Sink,
) {
    let current_player_position = game_state.player_position;
    let next_player_position = next_position(&direction, &current_player_position, game_state);

    let next_player_position_contents =
        game_state.grid[next_player_position.1 as usize][next_player_position.0 as usize];
    let current_player_position_contents =
        game_state.grid[current_player_position.1 as usize][current_player_position.0 as usize];

    if next_player_position_contents == '#' {
        play_sound(SoundType::Oof, sink);
        return;
    }
    if next_player_position_contents == ' ' {
        set_grid_cell(&mut game_state.grid, &next_player_position, '@');
    }
    if next_player_position_contents == '.' {
        set_grid_cell(&mut game_state.grid, &next_player_position, '+');
    }
    if next_player_position_contents == '$' || next_player_position_contents == '*' {
        let next_player_position_plusone =
            next_position(&direction, &next_player_position, game_state);
        let next_player_position_plusone_contents = game_state.grid
            [next_player_position_plusone.1 as usize][next_player_position_plusone.0 as usize];
        if next_player_position_plusone_contents == '$'
            || next_player_position_plusone_contents == '*'
            || next_player_position_plusone_contents == '#'
        {
            play_sound(SoundType::BarrelOof, sink);
            return;
        }

        if next_player_position_plusone_contents == ' ' {
            set_grid_cell(&mut game_state.grid, &next_player_position_plusone, '$');
        }
        if next_player_position_plusone_contents == '.' {
            set_grid_cell(&mut game_state.grid, &next_player_position_plusone, '*');
        }

        if next_player_position_contents == '$' {
            set_grid_cell(&mut game_state.grid, &next_player_position, '@');
        }
        if next_player_position_contents == '*' {
            set_grid_cell(&mut game_state.grid, &next_player_position, '+');
        }
    }
    if current_player_position_contents == '@' {
        set_grid_cell(&mut game_state.grid, &current_player_position, ' ');
    }
    if current_player_position_contents == '+' {
        set_grid_cell(&mut game_state.grid, &current_player_position, '.');
    }
    game_state.player_position = next_player_position;
    if record_as_move {
        game_state.moves.push(direction);
    }
    game_state
        .scores
        .entry(game_state.level.unwrap())
        .and_modify(|val| val.1 += 1)
        .or_insert((0, 0));
}

fn set_grid_cell(grid: &mut Vec<Vec<char>>, coords: &(i32, i32), contents: char) {
    grid[coords.1 as usize][coords.0 as usize] = contents;
}

fn next_position(
    direction: &MoveDirection,
    current_position: &(i32, i32),
    game_state: &GameState,
) -> (i32, i32) {
    match direction {
        MoveDirection::Up => (current_position.0, std::cmp::max(0, current_position.1 - 1)),
        MoveDirection::Right => (
            std::cmp::min(
                game_state.grid[current_position.1 as usize].len() as i32 - 1,
                current_position.0 + 1,
            ),
            current_position.1,
        ),
        MoveDirection::Down => (
            current_position.0,
            std::cmp::min(game_state.grid.len() as i32 - 1, current_position.1 + 1),
        ),
        MoveDirection::Left => (std::cmp::max(0, current_position.0 - 1), current_position.1),
    }
}

fn read_input(key: KeyEvent) -> Option<Command> {
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
        return Some(Command::Quit);
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('m') {
        return Some(Command::LevelChoose);
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('w') {
        return Some(Command::Move(MoveDirection::Up));
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('a') {
        return Some(Command::Move(MoveDirection::Left));
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('s') {
        return Some(Command::Move(MoveDirection::Down));
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('d') {
        return Some(Command::Move(MoveDirection::Right));
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('1') {
        return Some(Command::LevelSelect(Level::One));
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('2') {
        return Some(Command::LevelSelect(Level::Two));
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('3') {
        return Some(Command::LevelSelect(Level::Three));
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('4') {
        return Some(Command::LevelSelect(Level::Four));
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('5') {
        return Some(Command::LevelSelect(Level::Five));
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('r') {
        return Some(Command::Reset);
    }
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('b') {
        return Some(Command::ReverseMove);
    }
    return None;
}

fn play_sound(sound_type: SoundType, sink: &Sink) {
    let path = match sound_type {
        SoundType::Oof => "src\\oof.mp3",
        SoundType::BarrelMove => "src\\metal-moving.mp3",
        SoundType::BarrelOof => "src\\box-crash.mp3",
        _ => "",
    };

    let file = std::fs::File::open(path).unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
}
