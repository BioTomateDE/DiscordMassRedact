use rand::prelude::IndexedRandom;
use rand::{random_bool, rng};
use crate::emojis::EMOJIS;

mod wordlist;
mod channels;
mod emojis;
mod redact;

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}


// fn generate_redacted() -> String {
//     let word_count: usize = rand::random_range(4..20);
//     let mut buf: String = String::new();
//
//     let mut can_capitalize: bool = true;
//
//     for _ in 0..word_count {
//         let mut word: String = wordlist::WORDLIST.choose(&mut rng()).unwrap().to_string();
//         if can_capitalize && random_bool(0.153) {
//             word = capitalize(&word);
//         }
//         buf.push_str(&word);
//         can_capitalize = false;
//
//         if random_bool(0.187) {
//             let punctuation: &str = [".", ",", "?", "!"].choose(&mut rng()).unwrap();
//             loop {
//                 buf.push_str(punctuation);
//                 if random_bool(0.9) { break }
//             }
//             can_capitalize = true;
//         }
//
//         while random_bool(0.04) {
//             let emoji: &str = EMOJIS.choose(&mut rng()).unwrap();
//             buf.push_str(&format!(" :{emoji}:"));
//         }
//
//         buf.push(' ');
//     }
//
//     if random_bool(0.07) {
//         buf.push_str([".", "?", "!"].choose(&mut rng()).unwrap());
//     }
//
//     buf
// }

fn main() {
    // reqwest::Request::new(reqwest::Method::GET)
    println!("{}", generate_redacted());
}
