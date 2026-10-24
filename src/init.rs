use::std::fs;
use std::{fs::File, io::{BufWriter, Write}};

use crate::helper::{CHECKING_NUM, PEERS};

const LINE_SIZE: usize = 82;
const FIXED_FLAG: u16 = 0b100_0000_0000;
const CAND_ALL: u16 = 0b011_1111_1110;

const BYTE_LOOKUP: [u16; 256] = {                    //absolute abomination but very butiful in its unique way
    let mut t = [CAND_ALL; 256];
    let mut d: u8 = 1;
    while d <= 9 {
        t[(b'0' + d) as usize] = FIXED_FLAG | (1u16 << d);
        d += 1;
    }
    t
};

const CHARACTER_LOOKUP: [u8; 512] = {
    let mut t = [b'0'; 512];          // Standard: keine eindeutige Ziffer
    let mut d: u8 = 1;
    while d <= 9 {
        t[(1usize << d) & 511] = b'0' + d;
        d += 1;
    }
    t
};




#[inline]
fn parse_line(line: &[u8; 81]) -> ([u16; 81], u8) {
    let mut out = [CAND_ALL; 81];
    let mut set_counter: u8 = 0;
    for i in 0..81 {
        let cell = BYTE_LOOKUP[line[i] as usize];
        if cell >= CHECKING_NUM {
            set_counter += 1;
            out[i] = cell;
            let mask = !cell | FIXED_FLAG;
            for peer_index in PEERS[i] {
                out[peer_index as usize] &= mask; 
            }
        }
    }
    (out, set_counter)
}


pub(crate) fn load(path: &str) -> Vec<([u16; 81], u8)> {
    let bytes = fs::read(path).expect("Datei konnte nicht gelesen werden");
    let ind_of_first_nl = bytes.iter().position(|&b| b == b'\n').expect("found first newLine");

    let body = &bytes[&ind_of_first_nl+1..];
    
    body.chunks_exact(LINE_SIZE)
        .map(|chunk| {
            let line = chunk.first_chunk::<81>().unwrap();
            return parse_line(line)
        })
    .collect()
}



pub(crate) fn store(path: &str, sudokus: &[([u16; 81], u8)]) {
    let mut out = Vec::with_capacity(16 + sudokus.len() * LINE_SIZE);

    out.extend_from_slice(sudokus.len().to_string().as_bytes());
    out.push(b'\n');

    for sudoku in sudokus {
        for cell in sudoku.0 {
            out.push(CHARACTER_LOOKUP[(cell & CAND_ALL) as usize & 511]);
        }
        out.push(b'\n');
    }

    let mut file = BufWriter::new(
        File::create(path).expect("Datei konnte nicht erstellt werden"));
    file.write_all(&out).expect("Schreiben fehlgeschlagen");
    file.flush().expect("Flush fehlgeschlagen");
} 