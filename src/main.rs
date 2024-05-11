use std::fs;
use chrono;
use rug::{ops::Pow, Float};
use rayon::prelude::*;


//// PARAMS - START
const MAX_CACHE_SIZE: usize = 16 * 10i32.pow(3) as usize;
// const MAX_CACHE_SIZE: usize = 16 * 10i32.pow(6) as usize;
const INDEX: i32 = 9;
const BASE_16: &[char; 16] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

const CHAR_RANGE: std::ops::Range<u16> = 1..0x7FF;
const PRECISION: u32 = 100;

const HASH_FUNCTION: &str = "e**.1*";

fn hash(store: Float, value: u32) -> Float {
    // return store / Float::with_val(PRECISION, value);
    return store.pow(0.1) * Float::with_val(PRECISION, value);
}

//// PARAMS - end

#[derive(Clone, PartialEq)]
struct State {
    start: Float,
    current_pair_index: usize,
    byte_count: usize,
    history: Vec<char>,
}

impl State {
    fn new(start: Float, current_pair_index: usize, byte_count: usize, history: Vec<char>) -> Self {
        State {
            start,
            current_pair_index,
            byte_count,
            history,
        }
    }
}

struct Encoder {
    target_hex_pairs: Vec<Vec<char>>,
    cache_size: usize,
    old_cache: Vec<State>,
    new_cache: Vec<State>,
    possible_chars: Vec<char>,
}

impl Encoder {
    fn new(
        initial_state: State,
        target_hex_pairs: Vec<Vec<char>>,
        cache_size: usize,
    ) -> Self {
        let possible_chars = CHAR_RANGE
            .filter_map(|char| {
                let c = char as i32;
                if is_valid_char(c) {
                    Some(std::char::from_u32(c as u32).unwrap())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let clone = initial_state.clone();

        Encoder {
            target_hex_pairs,
            cache_size,
            old_cache: vec![clone],
            new_cache: Vec::new(),
            possible_chars,
        }
    }

    fn generate_next_states(&self, state: &State) -> Vec<State> {
        let mut next_states = Vec::new();
        let current_pair = &self.target_hex_pairs[state.current_pair_index];
    
        for &char in &self.possible_chars {
            let current_hash = create_hash(state.start.clone(), char as u32);
    
            // Checking conditions before any heavy operations like cloning
            if check_condition(state.start.clone(), &current_hash, &current_pair, INDEX as u32) {
                // Only clone when necessary, reducing clone operations
                let new_start = state.start.clone() + &current_hash[0] + &current_hash[1];
                let mut new_history = state.history.clone(); // Clone once, then modify
                new_history.push(char);
                let new_byte_count = byte_size(&new_history.iter().collect::<String>());
    
                if new_byte_count < 510 { 
                    next_states.push(State::new(
                        new_start,
                        state.current_pair_index + 1,
                        new_byte_count,
                        new_history,
                    ));
                }
            }
        }
    
        next_states
    }
    

    fn prune_cache(&mut self) {
        if self.new_cache.len() > self.cache_size {
            // println!("Pruning cache");
            let mut sorted_states: Vec<_> = self.new_cache.iter().cloned().collect();
            sorted_states.sort_by(|a, b| a.byte_count.cmp(&b.byte_count));

            self.new_cache = sorted_states.into_iter().take(self.cache_size).collect();
        }
    }

    fn encode(&mut self) -> Option<Vec<State>> {
        let mut completed = false;
        let mut counter = 0;
        while !completed {

            self.new_cache = self.old_cache.par_iter().flat_map(|state| {
                self.generate_next_states(state)
            }).filter(|state| state.byte_count > 0).collect();

            counter += 1;
            println!("{} {} of {} - {}", chrono::Utc::now().format("%d/%m/%Y %H:%M:%S"), counter, self.target_hex_pairs.len(), self.new_cache.len());

            self.prune_cache();

            if counter == self.target_hex_pairs.len() {
                completed = true;
                return Some(self.new_cache.clone());
            }
            self.old_cache.clear();
            std::mem::swap(&mut self.old_cache, &mut self.new_cache);
        }
        
        None 
    }
}


////// UTILS
 
fn byte_size(string: &str) -> usize {
    string.len()
}

fn is_valid_char(x: i32) -> bool {
    // 13 - carriage return
    // 36 - $
    // 92 - \
    // 96 - `
    // 127 - delete
    match x {
        13 | 92 | 96 | 127 => false,
        // 13 | 36 | 92 | 96 | 127 => false,
        _ => true,
    }
}

// get hex digit at dth position
fn get_hex_digit(x: Float, d: usize) -> char {

    let x = x.clone();
    let d = Float::with_val(PRECISION, d);

    let mut shift = Float::with_val(PRECISION, 1.0);
    let mut threshold = Float::with_val(PRECISION, 16.0);

    while x >= threshold {
        shift += Float::with_val(PRECISION, 1.0);
        threshold *= Float::with_val(PRECISION, 16.0);
    }

    if d == shift {
        return 'z';
    } else if d < shift {
        let digit = ((x.to_f32() as i32) >> (((shift - Float::with_val(PRECISION, 1.0) - d).to_f32() as i32) * Float::with_val(PRECISION, 4.0).to_f32() as i32)) & 15;
        return BASE_16[digit as usize] as char;
    } else {
        let adj_x = x * Float::with_val(PRECISION, 16.0).pow((d - shift));
        let ret = ((adj_x.to_f64() as usize) & 15) as usize;


        if (ret == 0) && (adj_x%Float::with_val(PRECISION, 1.0) == Float::with_val(PRECISION, 0.0)) {
            return 'z';
        } else {
            return BASE_16[ret as usize] as char;
        }
    }
}

// create two hash values for each pair to check if the condition is satisfied
fn create_hash(start: Float, x: u32) -> Vec<Float> {
    let h1 = hash(start.clone(), x);
    let new_hash = hash(start + h1.clone(), x);
    return vec![h1, new_hash];
}

fn check_condition(
    start: Float,
    hash_values: &Vec<Float>,
    pair: &&Vec<char>,
    y: u32
) -> bool {
    let mut digit = get_hex_digit(start.clone() + hash_values[0].clone(), y.try_into().unwrap());
    if digit != pair[0] {
        return false;
    }

    // handle when there is single character in pair
    if pair.len() == 1 {
        return true;
    }

    digit = get_hex_digit(start.clone() + hash_values[0].clone() + hash_values[1].clone(), y.try_into().unwrap());
    if digit != pair[1] {
        return false;
    }

    true
}

fn main() {
    let target_hex = "0f8fffae".to_string();
    // let target_hex = "f0f8fffaebd700ffff7fffd4f0fffff5f5dcffe4c4000000ffebcd0000ff8a2be2a52a2adeb8875f9ea07fff00d2691eff7f506495edfff8dcdc143c00ffff00008b008b8bb8860ba9a9a9006400a9a9a9bdb76b8b008b556b2fff8c009932cc8b0000e9967a8fbc8f483d8b2f4f4f2f4f4f00ced19400d3ff149300bfff6969696969691e90ffb22222fffaf0228b22ff00ffdcdcdcf8f8ffffd700daa520808080008000adff2f808080f0fff0ff69b4cd5c5c4b0082fffff0f0e68ce6e6fafff0f57cfc00fffacdadd8e6f08080e0fffffafad2d3d3d390ee90d3d3d3ffb6c1ffa07a20b2aa87cefa778899778899b0c4deffffe000ff0032cd32faf0e6ff00ff80000066cdaa0000cdba55d39370db3cb3717b68ee00fa9a48d1ccc71585191970f5fffaffe4e1ffe4b5ffdead000080fdf5e68080006b8e23ffa500ff4500da70d6eee8aa98fb98afeeeedb7093ffefd5ffdab9cd853fffc0cbdda0ddb0e0e6800080663399ff0000bc8f8f4169e18b4513fa8072f4a4602e8b57fff5eea0522dc0c0c087ceeb6a5acd708090708090fffafa00ff7f4682b4d2b48c008080d8bfd8ff634740e0d0ee82eef5deb3fffffff5f5f5ffff009acd3".to_string();
    // reverse target_hex
    let target_hex = target_hex.chars().rev().collect::<String>();
    
    // split target_hex into pairs of 2 characters which can be indexed 0 and 1
    let target_hex_pairs = target_hex.chars().collect::<Vec<_>>().chunks(2).map(|pair| pair.to_vec()).collect::<Vec<_>>();

    let initial_state = State::new(Float::with_val(PRECISION, 2.0), 0, 0, vec![]);
    let mut encoder = Encoder::new(initial_state, target_hex_pairs, MAX_CACHE_SIZE);

    if let Some(result) = encoder.encode() {
        println!("Encoding Complete:");
        // group result by byte count
        let mut result = result.into_iter().collect::<Vec<_>>();
        result.sort_by(|a, b| a.byte_count.cmp(&b.byte_count));
        // write to file first 15 results
        let result = result.into_iter().take(15).collect::<Vec<_>>();

        for (i, r) in result.iter().enumerate() {
            println!("Encoded Sequence {}: {}", i, r.history.iter().collect::<String>());
            println!("Byte Count {}: {}", i, r.byte_count);
            fs::write(
                format!("out/rs_{}.txt", i),
                format!(
                    "for(w=i=e=2;e+={HASH_FUNCTION}`-{}`.charCodeAt(i++/2);)w=e.toString(16)[{}]+w\n\n",
                    r.history.iter().collect::<String>(),
                    INDEX
                ),
            )
            .expect("Failed to write to file");
        }
    } else {
        println!("No valid encoding found.");
    }
}
