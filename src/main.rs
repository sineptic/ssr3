const EXAMPLE_TEXT: &str = r#"
13 task - `hi`
Algorithm:
1. `it exists`
2. `not bla bla`
3. - for `nouns, adjectives, etc.`:
     `asdf asdf`
   - for `qwerty`:
     `hjkl`
"#;

fn main() {
    let example = EXAMPLE_TEXT.trim();
    let parsed = parse_task(example);

    // println!("{parsed:#?}");
    for block in parsed {
        match block {
            Block::Text(inner) => {
                print!("{inner}");
            }
            Block::HiddenText(inner) => {
                print!("\x1b[1;3m{inner}\x1b[0;1;34m|\x1b[0m");
            }
        }
    }
    println!();
}

#[derive(Debug)]
enum Block {
    Text(String),
    HiddenText(String),
}
impl Block {
    fn grow(&mut self, ch: char) {
        match self {
            Block::Text(this) => {
                this.push(ch);
            }
            Block::HiddenText(this) => {
                this.push(ch);
            }
        }
    }
}

fn parse_task(text: &str) -> Vec<Block> {
    let mut answer = Vec::new();
    let mut block = Block::Text(String::new());
    for ch in text.chars() {
        if ch == '`' {
            match block {
                Block::Text(ref inner) => {
                    if !inner.is_empty() {
                        answer.push(block);
                    }
                    block = Block::HiddenText(String::new());
                }
                Block::HiddenText(_) => {
                    answer.push(block);
                    block = Block::Text(String::new());
                }
            }
            continue;
        }
        block.grow(ch);
    }
    match block {
        Block::Text(ref inner) => {
            if !inner.is_empty() {
                answer.push(block);
            }
        }
        Block::HiddenText(_) => {
            panic!("Unclosed hidden text field.");
        }
    }

    return answer;
}
