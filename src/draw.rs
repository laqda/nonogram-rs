extern crate lazy_static;
extern crate termion;


use crate::board::{Board, Cell, Line, Cursor, Status};
use std::io::{Write, stdout, stdin, StdoutLock};
use termion::{cursor, clear, style};
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::color::{Fg, Bg, Rgb, Black, White};
use termion::event::Key;
use termion::input::TermRead;
use std::cmp;

static MARGIN_VERTICAL: u16 = 2;
static MARGIN_HORIZONTAL: u16 = 4;

static CELL_WIDTH: u16 = 4;
static CELL_HEIGHT: u16 = 3;

static BOTTOM_BLOCK_HEIGHT: u16 = 6;

lazy_static! {
    static ref COLOR_DEFAULT: String = format!("{}{}", Bg(Black), Fg(Rgb(200, 200, 200)));
    static ref CURSOR_COLOR: String = format!("{}{}", Bg(Black), Fg(White));
    static ref GRID_COLOR: String = format!("{}{}", Bg(Black), Fg(Rgb(117, 117, 117)));
    static ref INDICATIONS_COLOR: String = format!("{}{}", Bg(Black), Fg(Rgb(180, 180, 180)));
    static ref INDICATIONS_CURRENT_COLOR: String = format!("{}{}", Bg(Black), Fg(Rgb(240, 240, 240)));
    static ref GRID_CELL_MARKED: String = format!("{}  {}{}", Bg(Rgb(180, 180, 180)), Fg(White), *GRID_COLOR);
    static ref GRID_CELL_EMPTY: String = format!("{}  {}{}", Bg(Black), Fg(Black), *GRID_COLOR);
    static ref GRID_CELL_NONE: String = format!("{}  {}{}", Bg(Rgb(80, 80, 80)), Fg(Black), *GRID_COLOR);
}


struct BoardDisplay {
    pub grid_width: usize,
    pub grid_height: usize,

    pub grid_margin_left: u16,
    pub grid_margin_right: u16,
    pub grid_margin_top: u16,
    pub grid_margin_bottom: u16,

    pub indications_max_char_space_needed_rows: usize,
    pub indications_max_char_space_needed_columns: usize,
}

impl BoardDisplay {
    fn goto_cell(&self, x: usize, y: usize, move_x: usize, move_y: usize) -> cursor::Goto {
        cursor::Goto(
            self.grid_margin_left + x as u16 * (CELL_WIDTH - 1) + move_x as u16,
            self.grid_margin_top + y as u16 * (CELL_HEIGHT - 1) + move_y as u16,
        )
    }
}

fn draw_row_indications(stdout: &mut RawTerminal<StdoutLock>, board_display: &BoardDisplay, row: &Line, position: usize, current: bool) {
    for (i, c) in row.get_indications_as_string().chars().into_iter().enumerate() {
        let goto = cursor::Goto(
            board_display.grid_margin_left - (i as u16 + 2),
            board_display.grid_margin_top + 1 + (CELL_HEIGHT - 1) * position as u16,
        );
        match current {
            true => write!(stdout, "{}{}{}{}{}", goto, style::Bold, &*INDICATIONS_CURRENT_COLOR, c, style::Reset).unwrap(),
            false => write!(stdout, "{}{}{}", goto, &*INDICATIONS_COLOR, c).unwrap(),
        };
    }
}

fn draw_column_indications(stdout: &mut RawTerminal<StdoutLock>, board_display: &BoardDisplay, column: &Line, position: usize, current: bool) {
    for (i, c) in column.get_indications_as_string().chars().into_iter().enumerate() {
        let goto = cursor::Goto(
            board_display.grid_margin_left + 1 + (CELL_WIDTH - 1) * position as u16,
            board_display.grid_margin_top - (i as u16 + 1),
        );
        match current {
            true => write!(stdout, "{}{}{}{}{}", goto, style::Bold, &*INDICATIONS_CURRENT_COLOR, c, style::Reset).unwrap(),
            false => write!(stdout, "{}{}{}", goto, &*INDICATIONS_COLOR, c).unwrap(),
        };
    }
}

fn get_cell_corner_top_left(_board_display: &BoardDisplay, x: usize, y: usize) -> &str {
    match (x, y) {
        (0, 0) => "┌",
        (0, _) => "├",
        (_, 0) => "┬",
        _ => "┼",
    }
}

fn get_cell_corner_top_right(board_display: &BoardDisplay, x: usize, y: usize) -> &str {
    match (x, y) {
        (x, 0) if x == board_display.grid_width - 1 => "┐",
        (x, _) if x == board_display.grid_width - 1 => "┤",
        (_, 0) => "┬",
        _ => "┼",
    }
}

fn get_cell_corner_bottom_left(board_display: &BoardDisplay, x: usize, y: usize) -> &str {
    match (x, y) {
        (0, y) if y == board_display.grid_height - 1 => "└",
        (_, y) if y == board_display.grid_height - 1 => "┴",
        (0, _) => "├",
        _ => "┼",
    }
}

fn get_cell_corner_bottom_right(board_display: &BoardDisplay, x: usize, y: usize) -> &str {
    match (x, y) {
        (x, y) if x == board_display.grid_width - 1 && y == board_display.grid_height - 1 => "┘",
        (_, y) if y == board_display.grid_height - 1 => "┴",
        (x, _) if x == board_display.grid_width - 1 => "┤",
        _ => "┼",
    }
}

fn draw_cell(stdout: &mut RawTerminal<StdoutLock>, board_display: &BoardDisplay, cell: &Cell) {
    let cell_value = match cell.status {
        Status::MARKED => &*GRID_CELL_MARKED,
        Status::EMPTY => &*GRID_CELL_EMPTY,
        Status::NONE => &*GRID_CELL_NONE,
    };

    write!(stdout, "{}{}──{}",
           board_display.goto_cell(cell.x, cell.y, 0, 0),
           get_cell_corner_top_left(&board_display, cell.x, cell.y),
           get_cell_corner_top_right(&board_display, cell.x, cell.y),
    ).unwrap();
    write!(stdout, "{}│{}│",
           board_display.goto_cell(cell.x, cell.y, 0, 1),
           cell_value
    ).unwrap();
    write!(stdout, "{}{}──{}",
           board_display.goto_cell(cell.x, cell.y, 0, 2),
           get_cell_corner_bottom_left(&board_display, cell.x, cell.y),
           get_cell_corner_bottom_right(&board_display, cell.x, cell.y),
    ).unwrap();
}

fn draw_cursor(stdout: &mut RawTerminal<StdoutLock>, board_display: &BoardDisplay, cursor: &Cursor) {
    write!(stdout, "{}", *CURSOR_COLOR).unwrap();
    write!(stdout, "{}┏━━┓", board_display.goto_cell(cursor.x, cursor.y, 0, 0)).unwrap();
    write!(stdout, "{}┃", board_display.goto_cell(cursor.x, cursor.y, 0, 1)).unwrap();
    write!(stdout, "{}┃", board_display.goto_cell(cursor.x, cursor.y, CELL_WIDTH as usize - 1, 1)).unwrap();
    write!(stdout, "{}┗━━┛", board_display.goto_cell(cursor.x, cursor.y, 0, 2)).unwrap();
}

fn remove_cursor(stdout: &mut RawTerminal<StdoutLock>, board_display: &BoardDisplay, cursor: &Cursor) {
    write!(stdout, "{}", *GRID_COLOR).unwrap();
    write!(stdout, "{}{}──{}",
           board_display.goto_cell(cursor.x, cursor.y, 0, 0),
           get_cell_corner_top_left(&board_display, cursor.x, cursor.y),
           get_cell_corner_top_right(&board_display, cursor.x, cursor.y)
    ).unwrap();
    write!(stdout, "{}│", board_display.goto_cell(cursor.x, cursor.y, 0, 1)).unwrap();
    write!(stdout, "{}│", board_display.goto_cell(cursor.x, cursor.y, CELL_WIDTH as usize - 1, 1)).unwrap();
    write!(stdout, "{}{}──{}",
           board_display.goto_cell(cursor.x, cursor.y, 0, 2),
           get_cell_corner_bottom_left(&board_display, cursor.x, cursor.y),
           get_cell_corner_bottom_right(&board_display, cursor.x, cursor.y)
    ).unwrap();
}

fn draw_bottom_block(stdout: &mut RawTerminal<StdoutLock>, board_display: &BoardDisplay) {
    write!(stdout, "{}{}{}", cursor::Goto(
        board_display.grid_margin_left + 2,
        board_display.grid_margin_top + 1 + (CELL_HEIGHT - 1) * board_display.grid_height as u16 + 1,
    ), &*INDICATIONS_COLOR, "Lives :").unwrap();
}

fn draw_lives(stdout: &mut RawTerminal<StdoutLock>, board_display: &BoardDisplay, lives: u16) {
    write!(stdout, "{}{}{}   ", cursor::Goto(
        board_display.grid_margin_left + 2 + 8,
        board_display.grid_margin_top + 1 + (CELL_HEIGHT - 1) * board_display.grid_height as u16 + 1,
    ), &*INDICATIONS_COLOR, lives).unwrap();
}

fn flush(stdout: &mut RawTerminal<StdoutLock>, board_display: &BoardDisplay) {
    // go to bottom corner right of the board
    write!(stdout, "{}", cursor::Goto(
        board_display.grid_margin_left + board_display.grid_margin_right + 1 + (CELL_WIDTH - 1) * board_display.grid_width as u16,
        board_display.grid_margin_top + board_display.grid_margin_bottom + 1 + (CELL_HEIGHT - 1) * board_display.grid_height as u16)
    ).unwrap();
    stdout.flush().unwrap();
}

pub fn draw(board: &mut Board) -> bool {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    let indications_max_char_space_needed_rows = cmp::max(40, board.grid.get_indications_max_char_space_needed_rows());
    let indications_max_char_space_needed_columns = cmp::max(20, board.grid.get_indications_max_char_space_needed_columns());

    let board_display = BoardDisplay {
        grid_width: board.grid.width,
        grid_height: board.grid.height,
        grid_margin_left: MARGIN_HORIZONTAL + indications_max_char_space_needed_rows as u16,
        grid_margin_right: MARGIN_HORIZONTAL,
        grid_margin_top: MARGIN_VERTICAL + indications_max_char_space_needed_columns as u16,
        grid_margin_bottom: MARGIN_VERTICAL + BOTTOM_BLOCK_HEIGHT,
        indications_max_char_space_needed_rows,
        indications_max_char_space_needed_columns,
    };

    write!(stdout, "{}{}", clear::All, *COLOR_DEFAULT).unwrap();

    for i in 0..board_display.grid_width {
        draw_column_indications(&mut stdout, &board_display, board.grid.get_column(i).unwrap(), i, i == board.cursor.x);
    };
    for j in 0..board_display.grid_height {
        draw_row_indications(&mut stdout, &board_display, board.grid.get_row(j).unwrap(), j, j == board.cursor.y);
    };

    write!(stdout, "{}", *GRID_COLOR).unwrap();
    for i in 0..board_display.grid_width {
        for j in 0..board_display.grid_height {
            let cell = board.grid.get_cell(i, j).unwrap();
            draw_cell(&mut stdout, &board_display, &cell);
            if cell.active {}
        };
    };

    draw_cursor(&mut stdout, &board_display, &board.cursor);

    draw_bottom_block(&mut stdout, &board_display);
    draw_lives(&mut stdout, &board_display, board.lives);

    flush(&mut stdout, &board_display);

    let error = false;
    while !error {
        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => {
                    write!(stdout, "{}{}", clear::All, *COLOR_DEFAULT).unwrap();
                    return true;
                }
                Key::Char('r') => {
                    write!(stdout, "{}{}", clear::All, *COLOR_DEFAULT).unwrap();
                    return false;
                }
                Key::Char('f') => {
                    match board.mark(board.cursor.x, board.cursor.y) {
                        Ok(()) => (),
                        Err(_) => {
                            draw_lives(&mut stdout, &board_display, board.lives);
                            flush(&mut stdout, &board_display);
                            return true
                        },
                    };
                    let cell = &board.grid.get_cell(board.cursor.x, board.cursor.y).unwrap();
                    draw_cell(&mut stdout, &board_display, cell);
                    draw_lives(&mut stdout, &board_display, board.lives);
                }
                Key::Char('v') => {
                    match board.none(board.cursor.x, board.cursor.y) {
                        Ok(()) => (),
                        Err(_) => {
                            draw_lives(&mut stdout, &board_display, board.lives);
                            flush(&mut stdout, &board_display);
                            return true
                        },
                    };
                    let cell = &board.grid.get_cell(board.cursor.x, board.cursor.y).unwrap();
                    draw_cell(&mut stdout, &board_display, cell);
                    draw_lives(&mut stdout, &board_display, board.lives);
                }
                Key::Left => {
                    let x = board.cursor.x;
                    remove_cursor(&mut stdout, &board_display, &board.cursor);
                    board.cursor.left();
                    draw_column_indications(&mut stdout, &board_display, board.grid.get_column(x).unwrap(), x, x == board.cursor.x);
                    draw_column_indications(&mut stdout, &board_display, board.grid.get_column(board.cursor.x).unwrap(), board.cursor.x, true);
                }
                Key::Right => {
                    let x = board.cursor.x;
                    remove_cursor(&mut stdout, &board_display, &board.cursor);
                    board.cursor.right();
                    draw_column_indications(&mut stdout, &board_display, board.grid.get_column(x).unwrap(), x, x == board.cursor.x);
                    draw_column_indications(&mut stdout, &board_display, board.grid.get_column(board.cursor.x).unwrap(), board.cursor.x, true);
                }
                Key::Up => {
                    let y = board.cursor.y;
                    remove_cursor(&mut stdout, &board_display, &board.cursor);
                    board.cursor.up();
                    draw_row_indications(&mut stdout, &board_display, board.grid.get_row(y).unwrap(), y, y == board.cursor.y);
                    draw_row_indications(&mut stdout, &board_display, board.grid.get_row(board.cursor.y).unwrap(), board.cursor.y, true);
                }
                Key::Down => {
                    let y = board.cursor.y;
                    remove_cursor(&mut stdout, &board_display, &board.cursor);
                    board.cursor.down();
                    draw_row_indications(&mut stdout, &board_display, board.grid.get_row(y).unwrap(), y, y == board.cursor.y);
                    draw_row_indications(&mut stdout, &board_display, board.grid.get_row(board.cursor.y).unwrap(), board.cursor.y, true);
                }

                _ => {}
            };
            draw_cursor(&mut stdout, &board_display, &board.cursor);
            flush(&mut stdout, &board_display);
        };
    };
    true
}
