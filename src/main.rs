use std::fs;
use chrono;

const MAX_CACHE_SIZE: usize = 16 * 10i32.pow(2) as usize;
// const MAX_CACHE_SIZE: usize = 16 * 10i32.pow(6) as usize;
const INDEX: i32 = 8;
const BASE_16: &[char; 16] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

fn hash(store: f64, value: u32) -> f64 {
    store.powf(0.1) * (value as f64)
}
#[derive(Clone, PartialEq)]
struct State {
    start: f64,
    current_pair_index: usize,
    byte_count: usize,
    history: Vec<char>,
}

impl State {
    fn new(start: f64, current_pair_index: usize, byte_count: usize, history: Vec<char>) -> Self {
        State {
            start,
            current_pair_index,
            byte_count,
            history,
        }
    }
}

struct Encoder {
    initial_state: State,
    target_hex: String,
    target_hex_pairs: Vec<String>,
    cache_size: usize,
    old_cache: Vec<State>,
    new_cache: Vec<State>,
    possible_chars: Vec<char>,
}

impl Encoder {
    fn new(
        initial_state: State,
        target_hex: String,
        target_hex_pairs: Vec<String>,
        cache_size: usize,
    ) -> Self {
        let possible_chars = (1..0xfff)
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
            initial_state,
            target_hex,
            target_hex_pairs,
            cache_size,
            old_cache: vec![clone],
            new_cache: Vec::new(),
            possible_chars,
        }
    }

    fn generate_next_states(&self, state: &State) -> Vec<State> {
        let mut next_states = Vec::with_capacity(MAX_CACHE_SIZE * 2);
        for char in &self.possible_chars {
            let current_pair = &self.target_hex_pairs[state.current_pair_index];
            let current_hash = create_hash(state.start as f64, *char as u32, current_pair.len());

            let current_pair_string = String::from_utf8(current_pair.as_bytes().to_vec())
                .expect("Failed to convert Vec<u8> to String");

            if check_condition(state.start, &current_hash, &current_pair_string, INDEX as u32) {
                let new_start = state.start + current_hash.iter().map(|x| *x as f64).sum::<f64>() as f64;
                let new_history = [state.history.clone(), vec![*char]].concat();
                let new_byte_count = byte_size(&new_history.iter().collect::<String>());

                let new_state = State {
                    start: new_start,
                    current_pair_index: state.current_pair_index + 1,
                    byte_count: new_byte_count,
                    history: new_history,
                };

                if new_state.byte_count < 650 {
                    next_states.push(new_state);
                }

                // if next_states.len() > 10 {
                //     next_states.sort_by(|a, b| a.byte_count.cmp(&b.byte_count));
                //     return next_states;
                // }
            }
        }

        next_states
    }

    fn prune_cache(&mut self) {
        if self.new_cache.len() > self.cache_size {
            println!("Pruning cache");
            let mut sorted_states: Vec<_> = self.new_cache.iter().cloned().collect();
            sorted_states.sort_by(|a, b| a.byte_count.cmp(&b.byte_count));

            self.new_cache = sorted_states.into_iter().take(self.cache_size).collect();
        }
    }

    fn encode(&mut self) -> Option<State> {
        let mut completed = false;
        let mut counter = 0;
        while !completed {

            // do it all in parallel

            // for state in &self.old_cache {
            //     let next_states = self.generate_next_states(state);

            //     for s in next_states {
            //         self.new_cache.push(s);
            //     }
            // }
            use rayon::prelude::*;

            self.new_cache = self.old_cache.par_iter().flat_map(|state| {
                self.generate_next_states(state)
            }).collect();

            counter += 1;
            println!("{} of {} - {}", counter, self.target_hex_pairs.len(), self.new_cache.len());

            self.prune_cache();

            
            // Check if we've reached the desired outcome
            for state in &self.new_cache {
                if state.current_pair_index >= self.target_hex_pairs.len() {
                    completed = true;
                    return Some(state.clone());
                }
            }

            if counter > self.target_hex_pairs.len() {
                break;
            }

            self.old_cache = self.new_cache.clone();
            self.new_cache.clear();
        }

        None
    }
}

fn byte_size(string: &str) -> usize {
    string.len()
}

fn is_valid_char(x: i32) -> bool {
    match x {
        13 | 36 | 92 | 96 => false,
        _ => true,
    }
}

fn get_hex_digit(x: f64, d: usize) -> char {
    // get dth digit of x in base 16 
    let x = x as f64;
    let d = d as f64;

    let mut shift = 1.0;
    let mut threshold = 16.0;
    

    while x >= threshold {
        shift += 1.0;
        threshold *= 16.0;
    }

    if d == shift {
        return 'z';
    } else if d < shift {
        let digit = ((x as u64) >> (((shift - 1.0 - d) * 4.0) as u64)) & 15;
        return BASE_16[digit as usize] as char;
    } else {
        let adj_x = (x as f64) * 16f64.powf(d - shift );
        let ret = (adj_x as u32 & 15) as usize;

        if (ret == 0) && (adj_x % 1.0 == 0.0) {
            return 'z';
        } else {
            return BASE_16[ret] as char;
        }
    }
}

fn create_hash(start: f64, x: u32, size: usize) -> Vec<f64> {
    let mut hash_values = vec![hash(start, x)];
    for i in 1..size {
        let sum: f64 = hash_values.iter().copied().sum();
        let new_hash = hash(start + sum, x);
        hash_values.push(new_hash);
    }
    hash_values
}

fn check_condition(
    start: f64,
    hash_values: &Vec<f64>,
    pair: &String,
    y: u32
) -> bool {
    for (i, expected) in pair.chars().enumerate() {
        let sum: f64 = hash_values.iter().take(i + 1).sum();
        let hex_digit = get_hex_digit(sum, y.try_into().unwrap());
        if hex_digit != expected {
            return false;
        }
    }
    true
}

fn main() {
    // let target_hex = "0f8fffae".to_string();
    let target_hex = "0f8fffaebd700ffff7fffd4f0fffff5f5dcffe4c4000000ffebcd0000ff8a2be2a52a2adeb8875f9ea07fff00d2691eff7f506495edfff8dcdc143c00ffff00008b008b8bb8860ba9a9a9006400a9a9a9bdb76b8b008b556b2fff8c009932cc8b0000e9967a8fbc8f483d8b2f4f4f2f4f4f00ced19400d3ff149300bfff6969696969691e90ffb22222fffaf0228b22ff00ffdcdcdcf8f8ffffd700daa520808080008000adff2f808080f0fff0ff69b4cd5c5c4b0082fffff0f0e68ce6e6fafff0f57cfc00fffacdadd8e6f08080e0fffffafad2d3d3d390ee90d3d3d3ffb6c1ffa07a20b2aa87cefa778899778899b0c4deffffe000ff0032cd32faf0e6ff00ff80000066cdaa0000cdba55d39370db3cb3717b68ee00fa9a48d1ccc71585191970f5fffaffe4e1ffe4b5ffdead000080fdf5e68080006b8e23ffa500ff4500da70d6eee8aa98fb98afeeeedb7093ffefd5ffdab9cd853fffc0cbdda0ddb0e0e6800080663399ff0000bc8f8f4169e18b4513fa8072f4a4602e8b57fff5eea0522dc0c0c087ceeb6a5acd708090708090fffafa00ff7f4682b4d2b48c008080d8bfd8ff634740e0d0ee82eef5deb3fffffff5f5f5ffff009acd32".to_string();
    let target_hex_pairs = target_hex.chars().collect::<Vec<_>>().chunks(2).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>();

    let initial_state = State::new(2.0, 0, 0, vec![]);
    let mut encoder = Encoder::new(initial_state, target_hex, target_hex_pairs, MAX_CACHE_SIZE);

    if let Some(result) = encoder.encode() {
        println!("Encoding Complete:");
        println!("Encoded Sequence: {}", result.history.iter().collect::<String>());
        println!("Byte Count: {}", result.byte_count);
        let timestamp = chrono::Utc::now().timestamp();
        fs::write(
            format!("{}.txt", timestamp),
            format!(
                "for(w='f',i=0,e=2;e+=e/`{}`.charCodeAt(i++/2);)w+=e.toString(16)[{}]",
                result.history.iter().collect::<String>(),
                INDEX
            ),
        )
        .expect("Failed to write to file");
    } else {
        println!("No valid encoding found.");
    }
}
