use crossterm::event::*;
use crossterm::terminal::ClearType;
use crossterm::{cursor, event, execute, queue, terminal};
use std::io;
use std::io::{stdout, Write};
use std::time::Duration;

const VERSION: &str = "0.1 beta";
struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not turn off raw mode");
        Output::clear_screen().expect("Error");
    }
}

struct Output {
    win_size: (usize, usize),
    contents: EditorContents,
    cursor_ctrl: CursorController,
}

impl Output {
    fn new() -> Self {
        let win_size: (usize, usize) = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self {
            win_size,
            contents: EditorContents::new(),
            cursor_ctrl: CursorController::new(win_size),
        }
    }

    fn move_cursor(&mut self, direction: KeyCode) {
        self.cursor_ctrl.move_cursor(direction);
    }

    fn move_10x(&mut self, direction: KeyCode) {
        self.cursor_ctrl.move_10x(direction);
    }

    fn clear_screen() -> io::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn refresh_screen(&mut self) -> io::Result<()> {
        queue!(self.contents, cursor::Hide, cursor::MoveTo(0, 0))?;
        self.draw_rows();
        let x = self.cursor_ctrl.cursor_x as u16;
        let y = self.cursor_ctrl.cursor_y as u16;
        queue!(self.contents, cursor::MoveTo(x, y), cursor::Show)?;
        self.contents.flush()
    }

    fn draw_rows(&mut self) {
        let screen_rows: usize = self.win_size.1;
        for i in 0..screen_rows {
            if i == screen_rows / 4 {
                // Welcome message
                self.draw_message("Welcome to Rudolf".to_string());
            } else if i == screen_rows / 4 + 1 {
                self.draw_message(format!("v{}", VERSION))
            } else {
                self.contents.push('~')
            }

            // TODO: build generic error handling module
            match queue!(self.contents, terminal::Clear(ClearType::UntilNewLine)) {
                Ok(t) => t,
                Err(e) => panic!("Problem during unwrap: {}", e.to_string()),
            }

            if i < screen_rows - 1 {
                self.contents.push_str("\r\n");
            }
        }
    }

    fn draw_message(&mut self, mut welcome: String) {
        let cols: usize = self.win_size.0;
        if welcome.len() > cols {
            welcome.truncate(cols);
        }

        // handle padding
        let mut padding: usize = (cols - welcome.len()) / 2;
        if padding != 0 {
            self.contents.push('~');
            padding -= 1
        }
        (0..padding).for_each(|_| self.contents.push(' '));

        self.contents.push_str(&welcome)
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> io::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}

struct Editor {
    reader: Reader,
    output: Output,
}

impl Editor {
    fn new() -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
        }
    }

    fn process_keypress(&mut self) -> io::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                kind: event::KeyEventKind::Press,
                state: event::KeyEventState::NONE,
            } => return Ok(false),
            KeyEvent {
                code: input_key @ (KeyCode::Up | KeyCode::Left | KeyCode::Down | KeyCode::Right),
                modifiers: KeyModifiers::SHIFT,
                kind: event::KeyEventKind::Press,
                state: event::KeyEventState::NONE,
            } => self.output.move_10x(input_key),
            KeyEvent {
                code: input_key @ (KeyCode::Up | KeyCode::Left | KeyCode::Down | KeyCode::Right),
                modifiers: KeyModifiers::NONE,
                kind: event::KeyEventKind::Press,
                state: event::KeyEventState::NONE,
            } => self.output.move_cursor(input_key),

            _ => {}
        }
        Ok(true)
    }

    fn run(&mut self) -> io::Result<bool> {
        self.output.refresh_screen()?;
        self.process_keypress()
    }
}

struct EditorContents {
    content: String,
}

impl EditorContents {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn push(&mut self, ch: char) {
        self.content.push(ch)
    }

    fn push_str(&mut self, string: &str) {
        self.content.push_str(string)
    }
}

impl io::Write for EditorContents {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(str) => {
                self.content.push_str(str);
                Ok(str.len())
            }
            Err(_) => {
                print!("Err");
                Err(io::ErrorKind::WriteZero.into())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out: Result<(), io::Error> = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

struct CursorController {
    cursor_x: usize,
    cursor_y: usize,
    cols: usize,
    rows: usize,
}

impl CursorController {
    fn new(win_size: (usize, usize)) -> CursorController {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            cols: win_size.0,
            rows: win_size.1,
        }
    }

    fn move_10x(&mut self, direction: KeyCode) {
        let vertical = self.rows/10;
        let horizontal = self.cols/10;
        match direction {
            KeyCode::Up => {
                self.cursor_y = self.cursor_y.saturating_sub(vertical);
            }
            KeyCode::Left => {
                self.cursor_x = self.cursor_x.saturating_sub(horizontal);
            }
            KeyCode::Down => {
                if self.cursor_y != self.rows - 1 {
                    self.cursor_y += vertical;
                }
            }
            KeyCode::Right => {
                if self.cursor_x != self.cols - 1 {
                    self.cursor_x += horizontal;
                }
            }
            _ => unimplemented!(),
        }
    }

    fn move_cursor(&mut self, direction: KeyCode) {
        match direction {
            KeyCode::Up => {
                self.cursor_y = self.cursor_y.saturating_sub(1);
            }
            KeyCode::Left => {
                self.cursor_x = self.cursor_x.saturating_sub(1);
            }
            KeyCode::Down => {
                if self.cursor_y != self.rows - 1 {
                    self.cursor_y += 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_x != self.cols - 1 {
                    self.cursor_x += 1;
                }
            }
            _ => unimplemented!(),
        }
    }
}

fn main() -> io::Result<()> {
    let _clean_up: CleanUp = CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor: Editor = Editor::new();
    while editor.run()? {}
    Ok(())
}
