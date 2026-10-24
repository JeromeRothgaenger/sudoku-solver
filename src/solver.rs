use crate::checker::check_conflict;
use crate::display::display_fields;
use crate::helper;
use crate::checker;
use crate::helper::ALL_UNITS;
use crate::helper::ALL_STRUCTS_VER;
use crate::helper::BLK_LIST;
use crate::helper::BLK_LIST_VER;
use crate::helper::CHECKING_NUM;
use crate::helper::COL_LIST;
use crate::helper::ROW_LIST;
use crate::helper::STRUCT_PEERS;
use crate::helper::lock_field;

const MAX_ITERATIONS: i16 = 100;


//TODO: cleanup code so it matches with solve_without_guesser()
pub(crate) fn solve(mut state: &mut [u16; 81], mut set_count: u8) -> i16 {
    let mut iteration :i16 = 0;
    let mut result = 0;

    while set_count < 81 && iteration < MAX_ITERATIONS {
        iteration += 1;

        
        let changed = solve_orphan_prisoners(&mut state);
        if changed > 0 {
            set_count += changed;
            continue;
        }
        // if elimnate_outsiders(&mut state) {continue;}
        if eliminate_pagans(&mut state) {continue;}
        // if eliminate_foreigners(&mut state) {continue;}

        if check_conflict(&mut state) {
            result = -2;
            break;
        } else if checker::check_correct(&state) {
            result = iteration;
            break;
        }
        let guesser_result = guesser(&mut state, set_count);
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
    return result;
}

pub fn solve_without_guesser(mut state: &mut [u16; 81], mut set_count: u8) -> i16 {
    let mut iteration :i16 = 0;

    while set_count < 81 && iteration < MAX_ITERATIONS {
        iteration += 1;
        let changed = solve_orphan_prisoners(&mut state);
        if changed > 0 {
            set_count += changed;
            continue;
        }
    
        // if elimnate_outsiders(&mut state) {continue;}
        if eliminate_pagans(&mut state) {continue;}
        // if eliminate_foreigners(&mut state) {continue;}
        break;
    }

    if checker::check_correct(& state) {
        return iteration;
    }else if check_conflict(&mut state) {
        return -2;
    } else if iteration >= MAX_ITERATIONS {
        println!("Max number of iterations reached: {}", iteration);
        return -3;
    } else {
        return -1;
    }
}

// orphans: only empty field in structure; prisoners: only possible field in a strucuture for a number
fn solve_orphan_prisoners(state: &mut [u16; 81]) -> u8 {
    let mut changed = 0;
    // orphans: only empty field in structure
    for unit in ALL_UNITS {
        let mut appeared_buffer: u16 = 0b_00_0000_0000; // list of every number that has appeared in the candidates
        let mut appeared_multiple_buffer: u16 = 0b_00_0000_0000;
        
        for index in unit {
            let field: u16 = state[index];
            if field < helper::CHECKING_NUM { // field is not set yet
                if field.count_ones() == 1 { // only one candidate left, set it
                    helper::lock_field(state, index);
                    changed += 1;
                } else {
                    appeared_multiple_buffer |= appeared_buffer & field;
                    appeared_buffer |= field;
                }
            }
        }

        let only_once_mask = appeared_buffer & !appeared_multiple_buffer; // contains candidates that appeared exactly once

        // prisoner: fields which are the only possible location for a number in a structure
        for index in unit {
            let field: u16 = state[index];
            if field < CHECKING_NUM && (field&only_once_mask).count_ones() == 1 { 
                helper::set_field(state, index, field&only_once_mask);
                changed += 1;            }
        }
    }
    
    changed
}

// find and remove candidates which are needed in a different structure
// equivalent to pointing pairs, pointing triples, box line reduction
// TODO: unify?
fn elimnate_outsiders(state: &mut [u16; 81]) -> bool { // intersection removal and ?pointing?
    let mut changed = false;
    let mut struct_index = 0;

    for structure in ALL_STRUCTS_VER {

        // get candidate list for struct and combine triples
        let structure_candidates = std::array::from_fn(|i| state[structure[i]]);
        let combined_triples = helper::combine_triples(&structure_candidates);
        let mut candidates_occuring_in_one_triple = helper::triple_xor(&combined_triples);
 
        
        // find candidates which occur only in one triple
        while candidates_occuring_in_one_triple != 0 {
            // get affected number and create bitmask
            let num = candidates_occuring_in_one_triple.trailing_zeros() as u16;
            let mask: u16 = 1 << num;

            // find location of affected number by searching through the three combined candidates
            for triple in 0..3 {
                if combined_triples[triple] & (1 << num) != 0{
                    // remove candidates
                    let affected_fields = STRUCT_PEERS[struct_index * 3 + triple];
                    for field in affected_fields {
                        if state[field as usize] & mask != 0 {
                            helper::remove_candidate(state, field as usize, mask);
                            changed = true;
                        }
                    }
                }
            }
            candidates_occuring_in_one_triple ^= mask;
            
        }
        struct_index += 1;
    }
    changed
}


// find pairs, triplets, etc. of candidates in a structure and remove them from other fields in the structure
// equivalent to naked pairs, triplets, etc.
fn eliminate_pagans(state: &mut [u16; 81]) -> bool {
    let mut changed = false;
    for unit in ALL_UNITS {
        let mut skip_fields_list: u16 = 0; // byte of fields that have already been checked for combinations
        let struct_candidates: [u16; 9] = std::array::from_fn(|i| state[unit[i]]);


        // add set fields to skip list
        for field_index in 0..9 {
            if struct_candidates[field_index] > helper::CHECKING_NUM {
                skip_fields_list |= 1 << field_index;
            }
        }
        // iterate start positions
        for start_field in 0..9 {
            // skip already checked fields
            if skip_fields_list & (1 << start_field) == 0 {
                skip_fields_list |= 1 << start_field;
                let mut candidate_combination = struct_candidates[start_field];
                let mut index = start_field;
                let mut combined_fields_list: u16 = 1 << start_field; // byte of fields that are part of the combination

                // create combintions
                while skip_fields_list.count_ones() < 9 && index <= 8 {
                    if skip_fields_list & (1 << start_field) != 0 {
                        index += 1;
                        continue;
                    }
                    let tmp_combination = candidate_combination | struct_candidates[index];
                    // combine
                    if (tmp_combination ^ candidate_combination).count_ones() <= 1 {
                        candidate_combination = tmp_combination;
                        skip_fields_list |= 1 << index;
                        combined_fields_list |= 1 << index;
                        index = start_field; // optimierbar
                    } else {
                        index += 1;
                    }
                }
                if candidate_combination.count_ones() <= combined_fields_list.count_ones() as u32 {
                    // iterate fields not contained in group
                    for field_index in 0..9 {
                        let global_index = unit[field_index];
                        
                        if combined_fields_list & (1 << field_index) == 0  && state[global_index] & candidate_combination != 0{
                            state[global_index] = (state[global_index] | candidate_combination)^candidate_combination;
                            changed = true;
                        }
                    }
                }
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

// equivalent to hidden pairs, triplets, etc.
fn eliminate_foreigners(state: &mut [u16; 81]) -> bool {
    let mut changed = false;
    for unit in ALL_UNITS {
        let skip_candidates_list: u16 = 0; // byte of candidates that have already been checked for combinations
        let struct_candidates: [u16; 9] = std::array::from_fn(|i| state[unit[i]]);

        // one byte for every number, 1 if the field corresponding to the bit position can contain the number
        let struct_candidate_fields = read_vertically_u16(&struct_candidates);


        // iterate candidates
        for start_candidate in 1..10 {
            // skip already checked candidates
            if skip_candidates_list & (1 << start_candidate) == 0 {
                let mut skip_list_copy = skip_candidates_list.clone();
                skip_list_copy |= 1 << start_candidate;
                let mut field_candidate_combination = struct_candidate_fields[start_candidate - 1];
                let mut candidate = start_candidate;
                let mut candidate_combination: u16 = 1 << start_candidate;

                // create combinations
                while skip_list_copy.count_ones() < 9 && candidate <= 9 && candidate_combination.count_ones() < field_candidate_combination.count_ones(){
                    if skip_list_copy & (1 << candidate) != 0 {
                        candidate += 1;
                        continue;
                    }
                    let tmp_combination = field_candidate_combination | struct_candidate_fields[candidate - 1];
                    // combine
                    if (tmp_combination ^ field_candidate_combination).count_ones() <= 1 {
                        field_candidate_combination = tmp_combination;
                        skip_list_copy |= 1 << candidate;
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
                    
                    // iterate candidates
                    while field_candidate_combination != 0 {
                        let structure_index = 8-field_candidate_combination.trailing_zeros();
                        let global_index = unit[structure_index as usize];
                        if state[global_index] < helper::CHECKING_NUM && state[global_index].count_ones() > number_of_affected_candidates {
                            state[global_index] &=  candidate_combination;
                            changed = true;
                        }
                        field_candidate_combination &= field_candidate_combination - 1;
                    }
                }
            }

        }
    }
    changed
}



fn guesser(state: &mut [u16; 81], mut set_count: u8) -> i16 {
    for (index, field) in state.clone().iter().enumerate() {
        if *field < helper::CHECKING_NUM {
            let mut states_generated = [0u16; 81];  //Buffer where all reached game states are ored together
            
            let base_state = state.clone();
            for candidate_index in 0..field.count_ones() as u8 {
                let mut state_clone = base_state.clone();

                let mask = helper::generate_mask(*field, candidate_index);
                helper::set_field(&mut state_clone, index, mask);
                
                let success = solve_without_guesser(&mut state_clone, set_count + 1);
         
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