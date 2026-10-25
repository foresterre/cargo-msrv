use console::{AnsiCodeIterator, measure_text_width};

const RESET: &str = "\x1b[0m";

pub struct Wrap {
    width: usize,
}

impl Wrap {
    pub fn new(width: usize) -> Self {
        Self { width }
    }

    pub fn lines(&self, text: &str) -> Vec<String> {
        let lines = text
            .split('\n')
            .flat_map(|line| self.wrap_line(line))
            .collect();

        continue_style_on_next_line(lines)
    }

    fn wrap_line(&self, line: &str) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current = String::new();
        let mut current_width = 0;
        let mut is_first_word = true;

        for word in line.split(' ').flat_map(|word| self.split_word(word)) {
            let word_width = measure_text_width(&word);

            if !is_first_word && current_width + 1 + word_width > self.width {
                lines.push(std::mem::take(&mut current));
                current_width = 0;
            } else if !is_first_word {
                current.push(' ');
                current_width += 1;
            }

            current.push_str(&word);
            current_width += word_width;
            is_first_word = false;
        }

        lines.push(current);
        lines
    }

    fn split_word(&self, word: &str) -> Vec<String> {
        if measure_text_width(word) <= self.width {
            return vec![word.to_string()];
        }

        let mut chunks = vec![String::new()];
        let mut chunk_width = 0;

        for (part, is_ansi) in AnsiCodeIterator::new(word) {
            if is_ansi {
                chunks.last_mut().unwrap().push_str(part);
                continue;
            }

            for c in part.chars() {
                let char_width = measure_text_width(c.encode_utf8(&mut [0; 4]));

                if chunk_width > 0 && chunk_width + char_width > self.width {
                    chunks.push(String::new());
                    chunk_width = 0;
                }

                chunks.last_mut().unwrap().push(c);
                chunk_width += char_width;
            }
        }

        chunks
    }
}

fn continue_style_on_next_line(lines: Vec<String>) -> Vec<String> {
    let mut active = String::new();

    lines
        .into_iter()
        .map(|line| {
            let mut styled = active.clone() + &line;

            for (part, is_ansi) in AnsiCodeIterator::new(&line) {
                match (is_ansi, part) {
                    (true, RESET | "\x1b[m") => active.clear(),
                    (true, code) => active.push_str(code),
                    (false, _) => {}
                }
            }

            if !active.is_empty() {
                styled.push_str(RESET);
            }

            styled
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_between_words() {
        assert_eq!(Wrap::new(7).lines("aaa bbb ccc"), vec!["aaa bbb", "ccc"]);
    }

    #[test]
    fn splits_words_longer_than_width() {
        assert_eq!(Wrap::new(3).lines("abcdefgh"), vec!["abc", "def", "gh"]);
    }

    #[test]
    fn keeps_newlines() {
        assert_eq!(Wrap::new(10).lines("a\n\nb"), vec!["a", "", "b"]);
    }

    #[test]
    fn contnue_style_on_next_line() {
        let lines = Wrap::new(3).lines("\x1b[2maaa bbb\x1b[0m");

        assert_eq!(lines, vec!["\x1b[2maaa\x1b[0m", "\x1b[2mbbb\x1b[0m"]);
    }
}
