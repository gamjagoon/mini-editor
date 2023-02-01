use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use crossterm::{event, terminal, execute, cursor, queue};
use std::time::Duration; /* add this line */
use std::io;
use std::io::*;
use crossterm::terminal::ClearType;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not disable raw mode")
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
            Err(_) => Err(io::ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

struct CursorController {
    cursor_x: usize,
    cursor_y: usize,
    screen_columns: usize,
    screen_rows: usize,
}

impl CursorController {
    fn new(win_size: (usize, usize)) -> CursorController {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            screen_columns: win_size.0,
            screen_rows:win_size.1,
        }
    }

    /* add this function */
    fn move_cursor(&mut self, direction: char) {
        match direction {
            'k' => {
                self.cursor_y = self.cursor_y.saturating_sub(1);
            }
            'h' => {
                if self.cursor_x != 0 {
                    self.cursor_x -= 1;
                }
            }
            'j' => {
                if self.cursor_y != self.screen_rows - 1 {
                    self.cursor_y += 1;
                }
            }
            'l' => {
                if self.cursor_x != self.screen_columns - 1 {
                    self.cursor_x += 1;
                }
            }
            _ => unimplemented!(),
        }
    }
}

struct Output {
    win_size: (usize, usize),
    editor_contents: EditorContents,
    cursor_controller: CursorController,
}

impl Output {
    fn new() -> Self{
        let win_size = terminal::size()
            .map(|(x,y) | (x as usize, y as usize)).unwrap();
        Self { 
            win_size,
            editor_contents: EditorContents::new(),
            cursor_controller: CursorController::new(win_size),
        }
    }

    fn clear_screen() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn draw_rows(&mut self) {
        let screen_rows = self.win_size.1;
        let screen_columns= self.win_size.0;
        for i in 0..screen_rows {
            if i == 0 {
                let mut welcom = format!("Minjae Editor -- verion {}", 1.1);
                if welcom.len() > screen_columns {
                    welcom.truncate(screen_columns);
                }
                self.editor_contents.push_str(&welcom);
            } else {
                self.editor_contents.push('~');
            }
            queue!(
                self.editor_contents,
                terminal::Clear(ClearType::UntilNewLine),
            ).unwrap();
            if i < screen_rows - 1 {
                self.editor_contents.push_str("\r\n");
            }
        }
    }

    fn refresh_screen(&mut self) -> crossterm::Result<()> {
        queue!(
            self.editor_contents,
            cursor::Hide,
            cursor::MoveTo(0, 0),
        )?;
        self.draw_rows();
        let cursor_x = self.cursor_controller.cursor_x;
        let cursor_y = self.cursor_controller.cursor_y;
        queue!(
            self.editor_contents, 
            cursor::MoveTo(cursor_x as u16,cursor_y as u16),
            cursor::Show
        )?;
        self.editor_contents.flush()
    }

    fn move_cursor(&mut self, direction : char) {
        self.cursor_controller.move_cursor(direction);
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(300))? {
                if let Event::Key(event) = event::read()? { /* add this line */
                    return Ok(event)
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
            reader:Reader,
            output:Output::new(),
        }
    }

    fn process_keypress(&mut self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::F(4),
                modifiers: event::KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => return Ok(false),
            KeyEvent {
                code: KeyCode::Char(val ),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => {
                match val {
                    'h'| 'j' | 'k' | 'l' => self.output.move_cursor(val),
                    _ => {}
                }
                }

            //end
            _ => {
            }
        }
        println!("{:?}", self.reader.read_key()?);
        Ok(true)
    }

    fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen()?;
        self.process_keypress()
    }
}

fn main() -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor = Editor::new();
    while editor.run()? {}
    Ok(())
}