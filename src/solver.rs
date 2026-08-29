use crate::checker::check_conflict;
use crate::display::display_candidates;
use crate::helper;
use crate::checker;
use crate::helper::ALL_UNITS;
use crate::helper::CHECKING_NUM;
use crate::helper::lock_field;

const MAX_ITERATIONS: i16 = 100;


pub(crate) fn solve(state: &mut [u16; 81]) -> i16 {
    let mut iteration :i16 = 0;
    let mut result = 0;

    while !checker::check_finish(state) && iteration < MAX_ITERATIONS {
        iteration += 1;

        let mut changed = solve_orphan_prisoners(state);
        if changed {continue;}
        changed |= eliminate_pagans(state);
        if changed {continue;}
        changed = elimnate_outsiders(state);
        if changed {continue;}
        changed = eliminate_foreigners(state);
        if changed {continue;}

        if check_conflict(state) {
            result = -2;
            display_candidates(state);
            break;
        } else if checker::check_correct(&state) {
            result = iteration;
            break;
        }
        let guesser_result = guesser(state);
        if guesser_result == 0 {
            continue;
        } 
        result += guesser_result;
        break;
    }
    if iteration >= MAX_ITERATIONS {
        println!("Max number of iterations reached: {}", iteration);
        result = -3;
    }
    //display_candidates(state);
    return result;
}

fn solve_without_guesser(state: &mut [u16; 81]) -> i16 {
    let mut iteration :i16 = 0;

    let mut result = 0;
    while !checker::check_finish(state) && iteration < MAX_ITERATIONS {
        iteration += 1;

        let mut changed = solve_orphan_prisoners(state);
        if changed {continue;}
        changed |= eliminate_pagans(state);
        if changed {continue;}
        changed = elimnate_outsiders(state);
        if changed {continue;}
        changed = eliminate_foreigners(state);
        if changed {continue;}

        if checker::check_correct(&state) {
            result = iteration;
            break;
        }else if check_conflict(state) {
            result = -2;
            break;
        } else if iteration >= MAX_ITERATIONS {
            println!("Max number of iterations reached: {}", iteration);
            result = -3;
        } else {
            result = -1;
            break;
        }
    }
    return result;
}

fn solve_orphan_prisoners(state: &mut [u16; 81]) -> bool {
    let mut changed = false;
    // rows
    for unit in ALL_UNITS {
        let mut appeared_once_buffer: u16 = 0b_00_0000_0000;
        let mut appeared_multiple_buffer: u16 = 0b_00_0000_0000;
        
        for index in unit {
            let field: u16 = state[index];
            if field < helper::CHECKING_NUM { // Prüfbit nicht gesetzt --> noch keine eingetragene lösung
                appeared_multiple_buffer |= appeared_once_buffer & field;
                appeared_once_buffer |= field;

                if field.count_ones() == 1 { // Prüfbit nicht gesetzt --> noch keine eingetragene lösung
                    helper::lock_field(state, index);
                    changed = true;
                }
            }
        }

        let only_once_mask = appeared_once_buffer & !appeared_multiple_buffer;

        for index in unit {
            let field: u16 = state[index];
            if (field&only_once_mask).count_ones() == 1 { 
                helper::set_field(state, index, field&only_once_mask);
                changed = true;
            }
        }
    }
    
    changed
}


// find and remove candidates which are needed in a different structure
fn elimnate_outsiders_structure(state: &mut [u16; 81], struct_coord_list: [[usize; 9]; 9], pattern_coord_list: [[usize; 9]; 9], coord_conv: fn(usize, usize) -> usize) -> bool { // intersection removal and ?pointing?
    let mut changed = false;
    
    for struct_index in 0..9 {

        // get candidate list for struct and combine triples
        let mut block_candidates = [0u16; 9];
        for (local_index, global_index) in struct_coord_list[struct_index].iter().enumerate() {
            block_candidates[local_index] = state[*global_index]
        }
        let combined_triples = helper::combine_triples(&block_candidates);
        let mut candidates_occuring_in_one_triple = helper::triple_xor(&combined_triples);
 
        
        // find candidates which occur only in one triple
        while candidates_occuring_in_one_triple != 0 {
            // get affected number and create bitmask
            let num = candidates_occuring_in_one_triple.trailing_zeros() as u16;
            let mask: u16 = 1 << num;

            // find location of affected number by searching through the three combined candidates
            for triple in 0..3 {
                if (combined_triples[triple] >> num) & 1 != 0{
                    // remove candidates
                    for field_index in pattern_coord_list[coord_conv(struct_index, triple)] {
                        if !(struct_coord_list[struct_index].contains(&field_index)) && state[field_index] & mask != 0 {
                            helper::remove_candidate(state, field_index, mask);
                            changed = true;
                        }
                    }
                }
            }
            candidates_occuring_in_one_triple ^= mask;
        }
    }
    changed
}

fn elimnate_outsiders(state: &mut [u16; 81]) -> bool{
    fn blk_to_row(struct_index: usize,triple: usize) -> usize {(struct_index/3)*3 + triple}
    fn blk_to_col(struct_index: usize,triple: usize) -> usize {(struct_index%3)*3 + triple}
    fn row_to_blk(struct_index: usize,triple: usize) -> usize {(struct_index/3)*3 + triple}
    fn col_to_blk(struct_index: usize,triple: usize) -> usize {struct_index/3 + triple * 3}

    let mut changed = false;

    changed |= elimnate_outsiders_structure(state, helper::BLK_LIST, helper::ROW_LIST, blk_to_row);
    changed |= elimnate_outsiders_structure(state, helper::BLK_LIST_VER, helper::COL_LIST, blk_to_col);
    changed |= elimnate_outsiders_structure(state, helper::ROW_LIST, helper::BLK_LIST, row_to_blk);
    changed |= elimnate_outsiders_structure(state, helper::COL_LIST, helper::BLK_LIST, col_to_blk);

    changed
}



fn eliminate_pagans(state: &mut [u16; 81]) -> bool {
    let mut changed = false;
    for index in 0..81 {
        if state[index] < CHECKING_NUM {
            if state[index].count_ones() == 1 {
                lock_field(state, index);
                changed = true
            }
        }
    }

    changed
}





const MASK: u16 = 0x03FE; // 0b0000_0011_1111_1110
const NUM_COLS: usize = 9;

/// Stack `words` as rows, reading only bits 1–9 vertically.
/// `cols[0]` = column of bit-1, `cols[8]` = column of bit-9.
fn read_vertically_u16(words: &[u16]) -> [u16; NUM_COLS] {
    let n = words.len();
    let mut cols = [0u16; NUM_COLS];

    for (row, &word) in words.iter().enumerate() {
        let word = word & MASK; // mask off bit 0 and bits 10-15
        for col in 0..NUM_COLS {
            let bit = (word >> (col as u16 + 1)) & 1; // col 0 = bit 1, col 8 = bit 9
            cols[col] |= (bit as u16) << (n - 1 - row);
        }
    }
    cols
}


fn eliminate_foreigners_structure(state: &mut [u16; 81], coord_list: [[usize; 9]; 9]) -> bool {
    let mut changed = false;
    for i in 0..9 {
        let struct_coords = coord_list[i];
        let skip_candidates_list: Vec<usize> = vec![];
        let struct_candidates: [u16; 9] = std::array::from_fn(|i| state[struct_coords[i]]);

        // one byte for every number, 1 if the field corresponding to the bit position can contain the number
        let struct_candidate_fields = read_vertically_u16(&struct_candidates);


        // iterate candidates
        for start_candidate in 1..10 {
            // skip already checked candidates
            if !skip_candidates_list.contains(&start_candidate) {
                let mut skip_list_copy = skip_candidates_list.clone();
                skip_list_copy.push(start_candidate);
                let mut field_candidate_combination = struct_candidate_fields[start_candidate - 1];
                let mut candidate = start_candidate;
                let mut candidate_combination: u16 = 1 << start_candidate;

                // create combinations
                while skip_list_copy.len() < 9 && candidate <= 9 && candidate_combination.count_ones() < field_candidate_combination.count_ones(){
                    if skip_list_copy.contains(&candidate) {
                        candidate += 1;
                        continue;
                    }
                    let tmp_combination = field_candidate_combination | struct_candidate_fields[candidate - 1];
                    // combine
                    if (tmp_combination ^ field_candidate_combination).count_ones() <= 1 {
                        field_candidate_combination = tmp_combination;
                        skip_list_copy.push(candidate);
                        candidate_combination |= 1 << candidate;
                        candidate = start_candidate; // optimierbar
                    } else {
                        candidate += 1;
                    }
                }

                let number_of_affected_candidates = candidate_combination.count_ones();
                let number_of_affected_fields = field_candidate_combination.count_ones();

                if number_of_affected_candidates == number_of_affected_fields && number_of_affected_candidates > 1 {
                    // find fields where foreigners need to be eliminated
                    let mut affected_fields = Vec::with_capacity(field_candidate_combination.count_ones() as usize);
                    while field_candidate_combination != 0 {
                        affected_fields.push(8 - (field_candidate_combination.trailing_zeros()) as usize);
                        field_candidate_combination &= field_candidate_combination - 1;
                    }
                    
                    // iterate candidates
                    for field_index in affected_fields {
                        let global_index = struct_coords[field_index];
                        if state[global_index] < helper::CHECKING_NUM && state[global_index].count_ones() > number_of_affected_candidates {
                            state[global_index] &=  candidate_combination;
                            changed = true;
                        }
                    }
                }
            }

        }
    }
    changed
}

fn eliminate_foreigners(state: &mut [u16; 81]) -> bool {
    let mut changed = false;
    // rows
    changed |= eliminate_foreigners_structure(state, helper::ROW_LIST);
    changed |= eliminate_foreigners_structure(state, helper::COL_LIST);
    changed |= eliminate_foreigners_structure(state, helper::BLK_LIST);
    changed
}



fn guesser(state: &mut [u16; 81]) -> i16 {
    for (index, field) in state.clone().iter().enumerate() {
        if *field < helper::CHECKING_NUM {
            let mut states_generated = [0u16; 81];  //Buffer where all reached game states are ored together
            
            let base_state = state.clone();
            for candidate in 0..field.count_ones() as u8 {
                let mut state_clone = base_state.clone();
                let mask = helper::generate_mask(*field, candidate);
                helper::set_field(&mut state_clone, index, mask);

                let success = solve_without_guesser(&mut state_clone);
         

                if success >= 0 {
                    *state = state_clone;
                    return success;

                } else if success == -2 {
                    helper::remove_candidate(state, index, mask);

                } else {
                    for fields_generated in 0..81 {
                        states_generated[fields_generated] |= state_clone[fields_generated];
                    }
                }
            }
            for fields_generated in 0..81 {
                state[fields_generated] &= states_generated[fields_generated];
            }
        }
    }
    return 0;
}