const KEYWORDS: &[&str] = &["true", "false", "null"];

pub fn highlight(source: &str) -> Vec<(&'static str, String)> {
    let characters: Vec<char> = source.chars().collect();
    let count = characters.len();
    let mut runs: Vec<(&'static str, String)> = Vec::new();
    let mut index = 0;
    while index < count {
        let current = characters[index];
        if current == '"' {
            let start = index;
            index += 1;
            while index < count {
                if characters[index] == '\\' && index + 1 < count {
                    index += 2;
                    continue;
                }
                let closing = characters[index] == '"';
                index += 1;
                if closing {
                    break;
                }
            }
            let class = if labels_a_field(&characters, index) {
                "tok-key"
            } else {
                "tok-string"
            };
            runs.push((class, characters[start..index].iter().collect()));
        } else if starts_number(&characters, index) {
            let start = index;
            index += 1;
            while index < count && continues_number(&characters, index) {
                index += 1;
            }
            runs.push(("tok-number", characters[start..index].iter().collect()));
        } else if current.is_alphabetic() {
            let start = index;
            while index < count && characters[index].is_alphanumeric() {
                index += 1;
            }
            let word: String = characters[start..index].iter().collect();
            let class = if KEYWORDS.contains(&word.as_str()) {
                "tok-keyword"
            } else {
                "tok-plain"
            };
            runs.push((class, word));
        } else {
            let start = index;
            index += 1;
            while index < count && !starts_token(&characters, index) {
                index += 1;
            }
            runs.push(("tok-plain", characters[start..index].iter().collect()));
        }
    }
    runs
}

fn starts_token(characters: &[char], index: usize) -> bool {
    let current = characters[index];
    current == '"' || current.is_alphabetic() || starts_number(characters, index)
}

fn starts_number(characters: &[char], index: usize) -> bool {
    let current = characters[index];
    if current.is_ascii_digit() {
        return true;
    }
    current == '-'
        && characters
            .get(index + 1)
            .is_some_and(|next| next.is_ascii_digit())
}

fn continues_number(characters: &[char], index: usize) -> bool {
    let current = characters[index];
    if current.is_ascii_digit() || current == '.' || current == 'e' || current == 'E' {
        return true;
    }
    (current == '+' || current == '-')
        && index
            .checked_sub(1)
            .and_then(|previous| characters.get(previous))
            .is_some_and(|previous| *previous == 'e' || *previous == 'E')
}

fn labels_a_field(characters: &[char], mut index: usize) -> bool {
    while index < characters.len() && characters[index].is_whitespace() {
        index += 1;
    }
    characters.get(index) == Some(&':')
}
