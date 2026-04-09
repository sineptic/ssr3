use std::time::Duration;

// const EXAMPLE_TEXT: &str = r#"
// 13 task - `hi`
// Algorithm:
// 1. `it exists`
// 2. `not bla bla`
// 3. - for `nouns, adjectives, etc.`:
//      `asdf asdf`
//    - for `qwerty`:
//      `hjkl` hihi
// "#;

const DIFFICULTY_ART: &str = "
╭──────────┬─────────┬─────────┬─────────╮
│ \x1b[31m1. again\x1b[0m │ \x1b[33m2. hard\x1b[0m │ \x1b[36m3. good\x1b[0m │ \x1b[32m4. easy\x1b[0m │
╰──────────┴─────────┴─────────┴─────────╯
";
const DIFFICULTY_ART_WIDTH: u16 = 42;

fn main() {
    let example = std::io::read_to_string(std::io::stdin()).unwrap();
    // let example = EXAMPLE_TEXT.trim();
    let mut stdout = std::io::stdout();
    enter_raw_terminal_mode(&mut stdout).unwrap();

    repeat_task(&mut stdout, &example).unwrap();

    reset_terminal(&mut stdout).unwrap();
}

/// Repeat task and get difficulty
fn repeat_task(mut stdout: impl std::io::Write, task: &str) -> std::io::Result<u8> {
    let parsed = parse_task(task);
    let mut blocks: Vec<DisplayBlock> = parsed.into_iter().map(Into::into).collect();

    let mut cursor = 0;
    for (i, block) in blocks.iter().enumerate() {
        match block {
            DisplayBlock::Text(_) => {}
            DisplayBlock::HiddenText { .. } => {
                cursor = i;
                break;
            }
        }
    }
    loop {
        display_blocks_interactive_mode(&mut stdout, &blocks, cursor)?;
        let event = crossterm::event::read()?;
        if handle_event_interactive_mode(event, &mut cursor, &mut blocks) {
            break;
        }
        // handle all available events
        while crossterm::event::poll(Duration::ZERO)? {
            let event = crossterm::event::read()?;
            if handle_event_interactive_mode(event, &mut cursor, &mut blocks) {
                break;
            }
        }
    }
    let difficulty = 'outer: loop {
        display_blocks_answer_overview(&mut stdout, &blocks)?;
        let event = crossterm::event::read()?;
        if let Some(difficulty) = handle_event_answer_overview(event) {
            break 'outer difficulty;
        }
        // handle all available events
        while crossterm::event::poll(Duration::ZERO)? {
            let event = crossterm::event::read()?;
            if let Some(difficulty) = handle_event_answer_overview(event) {
                break 'outer difficulty;
            }
        }
    };
    Ok(difficulty)
}

fn handle_event_answer_overview(event: crossterm::event::Event) -> Option<u8> {
    match event {
        crossterm::event::Event::Key(key) => match key.code {
            crossterm::event::KeyCode::Char('1') => Some(1),
            crossterm::event::KeyCode::Char('2') => Some(2),
            crossterm::event::KeyCode::Char('3') => Some(3),
            crossterm::event::KeyCode::Char('4') => Some(4),
            _ => None,
        },
        _ => None,
    }
}

fn enter_raw_terminal_mode(stdout: &mut impl std::io::Write) -> std::io::Result<()> {
    let res1 = crossterm::terminal::enable_raw_mode();
    let res2 = crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen);
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        _ = reset_terminal(&mut std::io::stdout());
        prev_hook(info);
    }));
    res1.and(res2)
}

fn reset_terminal(stdout: &mut impl std::io::Write) -> std::io::Result<()> {
    let res1 = crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen,);
    let res2 = crossterm::terminal::disable_raw_mode();
    res1.and(res2)
}

/// Returns `true` if should quit.
fn handle_event_interactive_mode(
    event: crossterm::event::Event,
    cursor: &mut usize,
    blocks: &mut [DisplayBlock],
) -> bool {
    if let crossterm::event::Event::Key(key_event) = event
        && key_event.is_press()
    {
        match key_event.code {
            crossterm::event::KeyCode::Char(ch) => {
                match &mut blocks[*cursor] {
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
                match &mut blocks[*cursor] {
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
                match &mut blocks[*cursor] {
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
            crossterm::event::KeyCode::Enter => {
                let mut new_cursor = *cursor + 1;
                loop {
                    if new_cursor >= blocks.len() {
                        return true;
                    }
                    if matches!(blocks[new_cursor], DisplayBlock::HiddenText { .. }) {
                        *cursor = new_cursor;
                        break;
                    }
                    new_cursor += 1;
                }
            }
            crossterm::event::KeyCode::Left => {
                *blocks[*cursor].field_cursor() = blocks[*cursor].field_cursor().saturating_sub(1);
            }
            crossterm::event::KeyCode::Right => {
                let user_input_len = blocks[*cursor].as_hidden_user_text().len();
                if *blocks[*cursor].field_cursor() < user_input_len {
                    *blocks[*cursor].field_cursor() += 1;
                }
            }
            crossterm::event::KeyCode::Tab | crossterm::event::KeyCode::Down => {
                let mut new_cursor = *cursor + 1;
                loop {
                    if new_cursor >= blocks.len() {
                        break;
                    }
                    if matches!(blocks[new_cursor], DisplayBlock::HiddenText { .. }) {
                        *cursor = new_cursor;
                        break;
                    }
                    new_cursor += 1;
                }
            }
            crossterm::event::KeyCode::BackTab | crossterm::event::KeyCode::Up => {
                if *cursor == 0 {
                    return false;
                }
                let mut new_cursor = *cursor - 1;
                loop {
                    if matches!(blocks[new_cursor], DisplayBlock::HiddenText { .. }) {
                        *cursor = new_cursor;
                        break;
                    }
                    if new_cursor == 0 {
                        break;
                    }
                    new_cursor -= 1;
                }
            }
            crossterm::event::KeyCode::Esc => return true,
            _ => {}
        }
    }
    false
}

fn display_blocks_answer_overview(
    mut stdout: impl std::io::Write,
    blocks: &Vec<DisplayBlock>,
) -> std::io::Result<()> {
    crossterm::queue!(
        stdout,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0),
        crossterm::cursor::Hide,
    )?;
    write!(stdout, "\x1b[0m")?;
    for block in blocks {
        match block {
            DisplayBlock::Text(inner) => {
                write!(stdout, "{}", inner.replace("\n", "\r\n"))?;
            }
            DisplayBlock::HiddenText {
                original_text,
                user_input,
                field_cursor: _,
            } => {
                let user_input = user_input.iter().collect::<String>();
                let user_input = user_input.trim();
                let original_text = original_text.trim();
                if original_text.is_empty() {
                    write!(stdout, "\x1b[3;32m<empty>\x1b[0m")?;
                } else {
                    write!(stdout, "\x1b[3;4;32m{}\x1b[0m", original_text)?;
                }
                if user_input != original_text {
                    write!(stdout, " ")?;
                    if user_input.is_empty() {
                        write!(stdout, "\x1b[3;33m<empty>\x1b[0m")?;
                    } else {
                        write!(stdout, "\x1b[3;4;33m{}\x1b[0m", user_input)?;
                    }
                }
            }
        }
    }
    let (columns, rows) = crossterm::terminal::size()?;
    for (i, line) in DIFFICULTY_ART.trim().lines().enumerate() {
        crossterm::queue!(
            stdout,
            crossterm::cursor::MoveTo((columns - DIFFICULTY_ART_WIDTH) / 2, rows - 5 + i as u16)
        )?;
        write!(stdout, "{}\r\n", line)?;
    }
    stdout.flush()?;
    Ok(())
}

fn display_blocks_interactive_mode(
    mut stdout: impl std::io::Write,
    blocks: &[DisplayBlock],
    cursor: usize,
) -> std::io::Result<()> {
    crossterm::queue!(
        stdout,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;
    write!(stdout, "\x1b[0m")?;
    let mut interactive_element_in_focus = false;
    for (i, block) in blocks.iter().enumerate() {
        match block {
            DisplayBlock::Text(inner) => {
                write!(stdout, "{}", inner.replace("\n", "\r\n"))?;
            }
            DisplayBlock::HiddenText {
                original_text: _,
                user_input,
                field_cursor,
            } => {
                if user_input.is_empty() && i != cursor {
                    write!(stdout, "\x1b[3m<empty>\x1b[0m")?;
                } else {
                    write!(
                        stdout,
                        "\x1b[3;4m{}",
                        user_input.iter().take(*field_cursor).collect::<String>()
                    )?;
                    if i == cursor {
                        interactive_element_in_focus = true;
                        crossterm::queue!(stdout, crossterm::cursor::SavePosition).unwrap();
                    }
                    write!(
                        stdout,
                        "{}",
                        user_input.iter().skip(*field_cursor).collect::<String>()
                    )?;
                    write!(stdout, "\x1b[0m")?;
                }
            }
        }
    }
    writeln!(stdout)?;
    if interactive_element_in_focus {
        crossterm::queue!(
            stdout,
            crossterm::cursor::RestorePosition,
            crossterm::cursor::Show
        )?;
    } else {
        crossterm::queue!(stdout, crossterm::cursor::Hide)?;
    }
    stdout.flush()?;
    Ok(())
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

    answer
}
