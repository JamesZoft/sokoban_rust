use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout},
    prelude::CrosstermBackend,
    style::Stylize,
    widgets::Paragraph,
    Terminal::{self},
};

#[derive(Clone)]
struct GameState {
    grid: Vec<Vec<char>>,
    player_position: (i32, i32),
    level: Level,
}

fn main() -> std::io::Result<()> {
    let mut terminal: Terminal<CrosstermBackend<std::io::Stdout>> = ratatui::init();
    let mut game_state = GameState {
        grid: vec!["Welcome!".chars().collect::<Vec<_>>()],
        player_position: (0, 0),
        level: Level::One,
    };
    loop {
        if let Event::Key(key) = event::read()? {
            if let Some(command) = read_input(key) {
                match command {
                    Command::Quit => break,
                    Command::Reset => {
                        (game_state.grid, game_state.player_position) =
                            start_level(game_state.level)
                    }
                    Command::LevelChoose => choose_level(&mut game_state),
                    Command::Move(direction) => player_move(direction, &mut game_state),
                    Command::LevelSelect(level) => {
                        (game_state.grid, game_state.player_position) = start_level(level)
                    }
                }
            }
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

fn choose_level(game_state: &mut GameState) {
    game_state.grid = vec!["Choose level:".chars().collect::<Vec<_>>(), vec!['1']];
}

fn start_level(_level: Level) -> (Vec<Vec<char>>, (i32, i32)) {
    return (
        vec![
            vec!['#', '#', '#', '#', '#'],
            vec!['#', ' ', ' ', ' ', '#'],
            vec!['#', '.', '$', '@', '#'],
            vec!['#', ' ', ' ', ' ', '#'],
            vec!['#', '#', '#', '#', '#'],
        ],
        (3, 2),
    );
}

fn player_move(direction: MoveDirection, game_state: &mut GameState) {
    let current_player_position = game_state.player_position;
    let next_player_position = next_position(&direction, &current_player_position, game_state);

    let next_player_position_contents =
        game_state.grid[next_player_position.1 as usize][next_player_position.0 as usize];
    let current_player_position_contents =
        game_state.grid[current_player_position.1 as usize][current_player_position.0 as usize];

    if next_player_position_contents == '#' {
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
            std::cmp::min(game_state.grid[0].len() as i32 - 1, current_position.0 + 1),
            current_position.1,
        ),
        MoveDirection::Down => (
            current_position.0,
            std::cmp::min(game_state.grid.len() as i32 - 1, current_position.1 + 1),
        ),
        MoveDirection::Left => (std::cmp::max(0, current_position.0 - 1), current_position.1),
    }
}

#[derive(PartialEq, Debug)]
enum MoveDirection {
    Up,
    Right,
    Down,
    Left,
}
#[derive(PartialEq, Clone, Copy)]
enum Level {
    One,
}
#[derive(PartialEq)]
enum Command {
    Quit,
    Move(MoveDirection),
    LevelChoose,
    LevelSelect(Level),
    Reset,
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
    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('r') {
        return Some(Command::Reset);
    }
    return None;
}
