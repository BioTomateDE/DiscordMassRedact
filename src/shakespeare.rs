use rand::random_range;

const RAW_QUOTES_CONTENT: &[u8] = include_bytes!("shakespeare.txt");

pub fn generate_shakespeare(length: usize) -> String {
    let file = RAW_QUOTES_CONTENT;
    let desired_quote_length = match length {
        0..=100 => 100,
        101..=300 => 300,
        _ => 2000,
    };

    let quotes_length = file.len();
    let mut index = quotes_length; // Will be reset in first iteration

    loop {
        if index + 1 >= quotes_length {
            index = random_range(0..quotes_length - 1);
        }

        index += 1;
        // Skip if in middle of line
        if file.get(index) != Some(&b'\n') {
            continue;
        }

        while file.get(index) == Some(&b'\n') {
            index += 1;
        }

        let mut result: String = String::new();

        // Check each full line
        loop {
            index += 1;

            if index >= quotes_length {
                break;
            }

            // TODO: @lenus why is this unused
            let character;
            if index + 50 < quotes_length {
                character = str::from_utf8(&file[index..index + 50]).unwrap();
            }

            // We want a line with only uppercase letters and a dot at the end
            if !file[index].is_ascii_uppercase() && file.get(index) != Some(&b'.') {
                break;
            }

            // If a dot is encountered, check if this is the end of the line
            if file.get(index) == Some(&b'.') && file.get(index + 1) == Some(&b'\n') {
                // Criteria were met. Now scan until next new line to obtain a quote
                index += 1;
                while !(file.get(index) == Some(&b'\n') && file.get(index + 1) == Some(&b'\n')) {
                    let character = format!("{:?}", file[index] as char);
                    result.push(file[index] as char);
                    index += 1;
                }
                break;
            }
        }

        if result.len() <= desired_quote_length && !result.is_empty() {
            return result;
        }
    }
}
