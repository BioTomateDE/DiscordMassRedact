use rand::prelude::IndexedRandom;
use rand::{random_bool, rng};
use crate::{capitalize, wordlist};
use crate::emojis::EMOJIS;

fn generate_redacted() -> String {
    let word_count: usize = rand::random_range(4..20);
    let mut buf: String = String::with_capacity(word_count * 8); // Pre-allocate space

    let mut can_capitalize = true;
    let mut sentence_length = 0;

    for _ in 0..word_count {
        // Choose word with occasional misspelling
        let mut word: String = if random_bool(0.03) {
            let mut w: String = wordlist::WORDLIST.choose(&mut rng()).unwrap().to_string();
            // Simple misspelling by adding/removing/changing a character
            let pos = rand::random_range(0..w.len());
            match rand::random_range(0..3) {
                0 => w.insert(pos, w.chars().nth(pos).unwrap_or('e')),
                1 => { if !w.is_empty() { w.remove(pos.min(w.len()-1)); } },
                _ => { w.replace_range(pos..=pos, &rand::random_range('a'..='z').to_string()); },
            }
            w
        } else {
            wordlist::WORDLIST.choose(&mut rng()).unwrap().to_string()
        };

        // Capitalization logic
        if can_capitalize {
            if random_bool(0.15) || sentence_length == 0 {
                word = capitalize(&word);
                can_capitalize = false;
            }
        }

        buf.push_str(&word);
        sentence_length += 1;

        // Punctuation with more natural distribution
        if random_bool(0.15 + (sentence_length as f64 * 0.02)) {
            let punctuation = match rand::random_range(0..100) {
                0..=70 => ".",      // 70% period
                71..=85 => ",",     // 15% comma
                86..=95 => "!",      // 10% exclamation
                _ => "?",            // 5% question
            };

            buf.push_str(punctuation);
            // Occasionally add multiple punctuation (but more realistically)
            if punctuation != "," && random_bool(0.3) {
                buf.push_str(punctuation);
                if random_bool(0.2) {
                    buf.push_str(punctuation);
                }
            }

            can_capitalize = punctuation != ",";
            if can_capitalize {
                sentence_length = 0;
            }
        }

        // Emojis with more natural placement
        if random_bool(0.05) {
            let emoji = EMOJIS.choose(&mut rng()).unwrap();
            // Sometimes put emoji before punctuation
            if random_bool(0.4) && !can_capitalize {
                buf.pop(); // Remove space if needed
                buf.push_str(&format!(" :{emoji}: "));
            } else {
                buf.push_str(&format!(":{emoji}: "));
            }
        } else {
            buf.push(' ');
        }
    }

    // Final punctuation if needed
    if !buf.ends_with(|c: char| ".!?".contains(c)) && random_bool(0.6) {
        buf.pop(); // Remove trailing space
        buf.push_str(match rand::random_range(0..100) {
            0..=80 => ".",   // 80% period
            81..=90 => "!",  // 10% exclamation
            _ => "?",       // 10% question
        });
    }

    // Occasionally add an ellipsis at start or end
    if random_bool(0.1) {
        if random_bool(0.5) {
            buf.insert_str(0, "...");
        } else {
            buf.push_str("...");
        }
    }

    buf
}

