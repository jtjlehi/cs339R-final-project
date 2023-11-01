# Final Project Proposal

The Project I'll be making is a sudoku solver.

## Basic Design/Concept

While the naive approach to writing a sudoku solver is by using back tracking, this can be quite boring, basic, and slow. Instead I'll be writing a sudoku solver which combines paper and pencil techniques like only candidate, and xy-chaining, and backtracking. When the techniques I've implemented cannot add anymore information, I'll use backtracking to add to the board. The guessing action I do will be move which gives me the most information, and then continue using the paper and pencil technique until I finish the board or find the board unsolvable and revert back to the last guess.

## Extensions

There are quite a few techniques known to solve the puzzle. I won't be implementing all of the techniques because there are to many and some will give only limited returns. The other thing I'd like to add is a way to prove, without filling out the board completely or finding a direct counterexample that a guess is incorrect.

## Similar Work

There are quite a few people who have implemented a backtracking algorithm to solve sudoku in rust (see [sudoku-solver-cli](https://github.com/Mortis66666/sudoku-solver-cli/tree/master). There is also a backtracking library on cargo. The backtracking library has sparse documentation which will take about as long to decipher as it would be to just implement it myself, so for now I'll implement backtracking myself.
