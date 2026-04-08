const EXAMPLE_TEXT: &str = r#"
13 task - `hi`
Algorithm:
1. `it exists`
2. `not bla bla`
3. - for `nouns, adjectives, etc.`:
     `asdf asdf`
   - for `qwerty`:
     `hjkl` hihi
"#;

fn main() {
    let example = EXAMPLE_TEXT.trim();
    let parsed = parse_task(example);
    let mut blocks: Vec<DisplayBlock> = parsed.into_iter().map(Into::into).collect();

    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode().unwrap();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen).unwrap();
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        _ = crossterm::terminal::disable_raw_mode();
        prev_hook(info);
    }));

    let mut cursor = 0;
    loop {
        crossterm::execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )
        .unwrap();
        print!("\x1b[0m");
        let mut interactive_element_in_focus = false;
        for (i, block) in blocks.iter().enumerate() {
            match block {
                DisplayBlock::Text(inner) => {
                    print!("{}", inner.replace("\n", "\r\n"));
                }
                DisplayBlock::HiddenText {
                    original_text: _,
                    user_input,
                    field_cursor,
                } => {
                    if user_input.is_empty() {
                        print!("\x1b[1;4;34m ");
                        if i == cursor {
                            interactive_element_in_focus = true;
                            crossterm::execute!(stdout, crossterm::cursor::SavePosition).unwrap();
                        }
                        print!("  \x1b[0m");
                    } else {
                        print!(
                            "\x1b[1;3;4m{}",
                            user_input.iter().take(*field_cursor).collect::<String>()
                        );
                        if i == cursor {
                            interactive_element_in_focus = true;
                            crossterm::execute!(stdout, crossterm::cursor::SavePosition).unwrap();
                        }
                        print!(
                            "{}",
                            user_input.iter().skip(*field_cursor).collect::<String>()
                        );
                        // print!("\x1b[0;1;34m|");
                        print!("\x1b[0m");
                    }
                }
            }
        }
        println!();
        if interactive_element_in_focus {
            crossterm::execute!(
                stdout,
                crossterm::cursor::RestorePosition,
                crossterm::cursor::Show
            )
            .unwrap();
        } else {
            crossterm::execute!(stdout, crossterm::cursor::Hide).unwrap();
        }
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                if key_event.is_press() {
                    match key_event.code {
                        crossterm::event::KeyCode::Char(ch) => {
                            match &mut blocks[cursor] {
                                DisplayBlock::Text(_) => {
                                    // there could be no interactive elements
                                }
                                DisplayBlock::HiddenText {
                                    original_text: _,
                                    user_input,
                                    field_cursor,
                                } => {
                                    user_input.insert(*field_cursor, ch);
                                    *field_cursor += 1;
                                }
                            };
                        }
                        crossterm::event::KeyCode::Backspace => {
                            match &mut blocks[cursor] {
                                DisplayBlock::Text(_) => {
                                    // there could be no interactive elements
                                }
                                DisplayBlock::HiddenText {
                                    original_text: _,
                                    user_input,
                                    field_cursor,
                                } => {
                                    if *field_cursor != 0 {
                                        user_input.remove(*field_cursor - 1);
                                        *field_cursor -= 1;
                                    }
                                }
                            };
                        }
                        crossterm::event::KeyCode::Delete => {
                            match &mut blocks[cursor] {
                                DisplayBlock::Text(_) => {
                                    // there could be no interactive elements
                                }
                                DisplayBlock::HiddenText {
                                    original_text: _,
                                    user_input,
                                    field_cursor,
                                } => {
                                    if *field_cursor < user_input.len() {
                                        user_input.remove(*field_cursor);
                                    }
                                }
                            };
                        }
                        crossterm::event::KeyCode::Enter => todo!(),
                        crossterm::event::KeyCode::Left => {
                            *blocks[cursor].field_cursor() =
                                blocks[cursor].field_cursor().saturating_sub(1);
                        }
                        crossterm::event::KeyCode::Right => {
                            let user_input_len = blocks[cursor].as_hidden_user_text().len();
                            if *blocks[cursor].field_cursor() < user_input_len {
                                *blocks[cursor].field_cursor() += 1;
                            }
                        }
                        crossterm::event::KeyCode::Tab | crossterm::event::KeyCode::Down => {
                            let mut new_cursor = cursor + 1;
                            loop {
                                if new_cursor >= blocks.len() {
                                    break;
                                }
                                if matches!(blocks[new_cursor], DisplayBlock::HiddenText { .. }) {
                                    cursor = new_cursor;
                                    break;
                                }
                                new_cursor += 1;
                            }
                        }
                        crossterm::event::KeyCode::BackTab | crossterm::event::KeyCode::Up => {
                            if cursor == 0 {
                                continue;
                            }
                            let mut new_cursor = cursor - 1;
                            loop {
                                if matches!(blocks[new_cursor], DisplayBlock::HiddenText { .. }) {
                                    cursor = new_cursor;
                                    break;
                                }
                                if new_cursor == 0 {
                                    break;
                                }
                                new_cursor -= 1;
                            }
                        }
                        crossterm::event::KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen,).unwrap();
    crossterm::terminal::disable_raw_mode().unwrap();
}

enum DisplayBlock {
    Text(String),
    HiddenText {
        original_text: String,
        user_input: Vec<char>,
        field_cursor: usize,
    },
}

impl DisplayBlock {
    fn as_hidden_user_text(&self) -> &[char] {
        match self {
            Self::HiddenText {
                user_input: user, ..
            } => user.as_slice(),
            _ => panic!("as_hidden_user_text is not available for this block"),
        }
    }
    fn field_cursor(&mut self) -> &mut usize {
        match self {
            Self::HiddenText { field_cursor, .. } => field_cursor,
            _ => panic!("field_cursor is not available for this block"),
        }
    }
}
impl From<Block> for DisplayBlock {
    fn from(value: Block) -> Self {
        match value {
            Block::Text(text) => DisplayBlock::Text(text),
            Block::HiddenText(text) => DisplayBlock::HiddenText {
                original_text: text,
                user_input: Vec::new(),
                field_cursor: 0,
            },
        }
    }
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
