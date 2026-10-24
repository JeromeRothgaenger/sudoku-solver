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
const START_PUZZLE: usize = 49152; // not 0
const PUZZEL_AMOUNT: usize = 49153;

fn main() -> io::Result<()> {
    //solve_one(START_PUZZLE);
    solve_all();
    //benchmark();
    Ok(())
}



fn solve_one(index: usize) {
    let start = Instant::now();
    let mut sudokus = load(INPUT_PATH);
    let count = sudokus[index].1;
    solver::solve(&mut sudokus[index].0, count);

    store(OUTPUT_PATH, &sudokus);
    let duration = start.elapsed().as_micros();

    #[cfg(debug_assertions)] {
        display::display_fields(&sudokus[index].0);
        display::display_candidates(&sudokus[index].0);
    }

    if !check_correct(&sudokus[index].0) {
        println!("Sudoku {} is wrong", index);
    }

    println!("Time elapsed: {:?}ms, avg time: {}mus", duration as f64/1000 as f64, duration/ PUZZEL_AMOUNT as u128);  
}



fn solve_all() {
 
    let start = Instant::now();
    let mut sudokus = load(INPUT_PATH);

    sudokus.par_chunks_mut(256).for_each(|chunk| {
        for sudoku in chunk {
            solver::solve(&mut sudoku.0, sudoku.1);
        }
    });

    store(OUTPUT_PATH, &sudokus);
    let duration = start.elapsed().as_micros();

    for index in 0..PUZZEL_AMOUNT {
        if !check_correct(&sudokus[index].0) {
            display::display_fields(&sudokus[index].0);
            panic!("Sudoku {} is wrong", index);
        }
    }

    println!("Time elapsed: {:?}ms, avg time: {:.3}mus", duration as f64/1000 as f64, duration as f64/ PUZZEL_AMOUNT as f64);
  
}




const BENCHMARK_MULTIPLIER: usize = 10;

//benchmark
fn benchmark() {
    let sudokus_main = load(INPUT_PATH);

    let start = Instant::now();
    for _ in 0..BENCHMARK_MULTIPLIER {
        let mut sudokus = sudokus_main.clone();
        for sudoku in sudokus.iter_mut() {
            solver::solve(&mut sudoku.0, sudoku.1);
        }
    }

    let duration = start.elapsed().as_micros();
    println!("Time elapsed: {:?}s, avg time: {}mus", duration/1000000, duration/(PUZZEL_AMOUNT * BENCHMARK_MULTIPLIER) as u128);
}

