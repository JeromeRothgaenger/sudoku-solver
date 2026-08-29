#![allow(dead_code)]

use std::{time::Instant};
use std::io;

use rayon::iter::ParallelIterator;
use rayon::slice::ParallelSliceMut;

use crate::checker::check_correct;
use crate::init::{load, store};



mod display;
mod helper;
mod solver;
mod checker;
mod init;

const INPUT_PATH: &'static str = "all_17_clue_sudokus.txt";
const OUTPUT_PATH: &'static str ="sudokus_out.txt";
const START_PUZZLE: usize = 1; // not 0
const PUZZEL_AMOUNT: usize = 49153;






fn main() {
 
    let start = Instant::now();
    let mut sudokus = load(INPUT_PATH);

    sudokus.par_chunks_mut(256).for_each(|chunk| {
        for sudoku in chunk {
            solver::solve(sudoku);
        }
    });

    store(OUTPUT_PATH, &sudokus);
    let duration = start.elapsed().as_micros();

    for sudoku in sudokus {
        if !check_correct(&sudoku) {
            panic!("sudoku wrong :(")
        }
    }

    println!("Time elapsed: {:?}ms, avg time: {}mus", duration as f64/1000 as f64, duration/ PUZZEL_AMOUNT as u128);
  
}



/*
const BENCHMARK_MULTIPLIER: usize = 50;

//benchmark
fn main() {
    let sudokus_main = load(INPUT_PATH);

    let start = Instant::now();
    for _ in 0..BENCHMARK_MULTIPLIER {
        let mut sudokus = sudokus_main.clone();
        for sudoku in sudokus.iter_mut() {
            solver::solve( sudoku);
        }
    }

    let duration = start.elapsed().as_micros();
    println!("Time elapsed: {:?}s, avg time: {}mus", duration/1000000, duration/(PUZZEL_AMOUNT * BENCHMARK_MULTIPLIER) as u128);
}
*/
