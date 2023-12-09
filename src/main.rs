use anyhow::Result;
use final_project::Board;
use std::{env, fs, process};

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
fn solve(lines: Vec<Vec<Option<u8>>>) -> Result<[[Option<usize>; 9]; 9]> {
    Ok(match Board::build(lines)?.solve() {
        Ok(board) => board.into(),
        Err(why) => Err(why)?,
    })
}
fn write_file(board: [[Option<usize>; 9]; 9]) -> Result<()> {
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
fn read_file() -> Result<Vec<Vec<Option<u8>>>> {
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
