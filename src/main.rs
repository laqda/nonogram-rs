#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;

mod board;
mod draw;

fn main() {
    let mut exit = false;
    while !exit {
        let mut board = board::Board::new(20, 20);
        exit = draw::draw(&mut board);
    }
}
