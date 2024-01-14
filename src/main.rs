use crossterm::{cursor, event, execute, terminal, queue};
use crossterm::event::*; 
use crossterm::terminal::ClearType; 
use std::time::Duration;
use std::io::{stdout, Write};
use std::io;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not turn off raw mode");
        Output::clear_screen().expect("Error");
    }
}


struct Output{
    win_size: (usize, usize),
    contents: EditorContents
}

impl Output {
    fn new() -> Self {
        let win_size: (usize, usize) = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap(); 
        Self { win_size,
        contents: EditorContents::new() }
    }


    fn clear_screen() -> io::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }
    
    fn refresh_screen(&mut self) -> io::Result<()> { /* modify */
        queue!(self.contents, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?; /* add this line*/
        self.draw_rows();
        queue!(self.contents, cursor::MoveTo(0, 0))?; /* modify */
        self.contents.flush()
    }
    fn draw_rows(&mut self) { /* modify */
        let screen_rows = self.win_size.1;
        for i in 0..screen_rows {
            self.contents.push('~'); /* modify */
            if i < screen_rows - 1 {
                self.contents.push_str("\r\n"); /* modify */
            }
        }
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
    output: Output
}

impl Editor {
    fn new() -> Self {
        Self { 
            reader: Reader,
            output: Output::new(),
        }
    }

    fn process_keypress(&self) -> io::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                kind: event::KeyEventKind::Press,
                state: event::KeyEventState::NONE,
            } => return Ok(false),
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
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => {
                print!("Err");
                Err(io::ErrorKind::WriteZero.into())}
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out: Result<(), io::Error> = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

fn main() -> io::Result<()> {
    let _clean_up: CleanUp = CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor: Editor = Editor::new();
    while editor.run()? {};
    Ok(())
}
