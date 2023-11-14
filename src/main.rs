use final_project::{Board, BoardState, UpdateError};
use std::{env, error::Error, fs, process};

fn main() {
    match read_file().and_then(solve).and_then(write_file) {
        Ok(()) => {
            println!("we solved a mystery")
        }
        Err(why) => {
            println!("error: {why:?}");
            process::exit(1)
        }
    }
}
fn solve(lines: Vec<Vec<Option<u8>>>) -> Result<[[Option<usize>; 9]; 9], Box<dyn Error>> {
    Ok(match Board::build(lines)?.solve() {
        BoardState::Finished(board) => board.into(),
        BoardState::Err(why) => Err(match why {
            UpdateError::InvalidConcrete => "found an  incorrect concrete value",
            UpdateError::InvalidCellVal => "found an invalid cell value",
            UpdateError::MultipleConcrete => {
                "tried to set a concrete value where there already was one"
            }
            UpdateError::InitError => "we didn't get past take off",
        })?,
        _ => Err("didn't finish")?,
    })
}
fn write_file(board: [[Option<usize>; 9]; 9]) -> Result<(), Box<dyn Error>> {
    let file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("out.csv")?;
    let mut writer = csv::Writer::from_writer(file);
    for line in board {
        writer.serialize(line)?;
    }
    writer.flush()?;
    Ok(())
}
fn read_file() -> Result<Vec<Vec<Option<u8>>>, Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    let file_name = &args[1];
    let file = fs::OpenOptions::new().read(true).open(file_name)?;
    Ok(csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(file)
        .deserialize()
        .collect::<Result<Vec<_>, _>>()?)
}
