# Sudoku Solver

A CLI tool that reads in sudoku files and solves them, generating a new file

## Use

Either generate the binary or use cargo to run the program. The program takes the filename as the input. 

`cargo run sudoku.csv`

## File Format

input files should be in a csv file. there are 9 rows and 9 columns. For cells that aren't filled in yet, leave them blank. See the `example.csv` for an example.

