#![allow(dead_code)]
use std::{fs, io, path::PathBuf, sync::mpsc::Sender};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use uuid::Uuid;

use crate::editor::{BufferEvent, EditorEvent};

pub struct Line {
    pub start: usize,
    pub end: usize,
}

impl Line {
    pub fn len(&self) -> usize {
        self.end - self.start + 1
    }
}

pub struct BufferData {
    pub data: Vec<char>,
    pub lines: Vec<Line>,
    pub cursor: usize,
    prev_cursor_offset: Option<usize>,
}

impl BufferData {
    pub fn new() -> Self {
        let mut buf_data = Self {
            data: vec![],
            lines: vec![],
            cursor: 0,
            prev_cursor_offset: None,
        };
        buf_data.recalculate_lines();
        buf_data
    }

    pub fn from(data: String) -> Self {
        let data = data
            .chars()
            .map(|b| b as char)
            .filter(|c| *c != '\r')
            .collect();

        let mut buf_data = Self {
            data,
            lines: vec![],
            cursor: 0,
            prev_cursor_offset: None,
        };
        buf_data.recalculate_lines();
        buf_data
    }

    pub fn recalculate_lines(&mut self) {
        let mut previous_begining = 0;
        self.lines.clear();

        for (i, ch) in self.data.iter().enumerate() {
            if *ch == '\n' {
                self.lines.push(Line {
                    start: previous_begining,
                    end: i,
                });
                previous_begining = i + 1;
            }
        }

        assert!(self.data.len() >= previous_begining);
        let end = if previous_begining == self.data.len() {
            previous_begining
        } else {
            self.data.len() - 1
        };

        self.lines.push(Line {
            start: previous_begining,
            end,
        });
    }

    pub fn current_line(&self) -> usize {
        let mut current_line = 0;

        for Line { start, end } in self.lines.iter() {
            if *start <= self.cursor && *end >= self.cursor {
                return current_line;
            } else {
                current_line += 1;
            }
        }

        assert!(self.cursor == self.data.len());
        current_line - 1
    }

    pub fn move_cursor_right(&mut self, dx: usize) {
        if self.cursor + dx <= self.data.len() {
            self.cursor += dx;
        }

        self.prev_cursor_offset = None;
    }

    pub fn move_cursor_left(&mut self, dx: usize) {
        if self.cursor >= dx {
            self.cursor -= dx;
        }

        self.prev_cursor_offset = None;
    }

    pub fn move_cursor_up(&mut self, dy: usize) {
        let mut current_line = self.current_line();

        if current_line >= dy {
            let line = &self.lines[current_line];
            let mut x_offset = match self.prev_cursor_offset {
                Some(offset) => offset,
                None => self.cursor - line.start,
            };

            current_line -= dy;

            let line = &self.lines[current_line];

            if x_offset >= line.len() {
                self.prev_cursor_offset = Some(x_offset);
                x_offset = line.len() - 1;
            }

            self.cursor = line.start + x_offset;
        }
    }

    pub fn move_cursor_down(&mut self, dy: usize) {
        let mut current_line = self.current_line();

        if current_line + dy < self.lines.len() {
            let line = &self.lines[current_line];
            let mut x_offset = match self.prev_cursor_offset {
                Some(offset) => offset,
                None => self.cursor - line.start,
            };

            current_line += dy;

            let line = &self.lines[current_line];

            if x_offset >= line.len() {
                self.prev_cursor_offset = Some(x_offset);
                x_offset = line.len() - 1;
            }

            self.cursor = line.start + x_offset;
        }
    }

    pub fn insert_ch(&mut self, ch: char) {
        self.data.insert(self.cursor, ch);
        self.cursor += 1;
    }

    /// Same as backspace key pressed
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.data.remove(self.cursor);
        }
    }

    /// Same as delete key pressed
    pub fn delete(&mut self) {
        if self.cursor < self.data.len() {
            self.data.remove(self.cursor);
        }
    }

    pub fn digits_in_line_num(&self) -> usize {
        let mut max = self.lines.len();
        let mut digits = 1; // start with a small gap
        while max > 0 {
            digits += 1;
            max = max.saturating_div(10);
        }
        digits
    }
}

// EditorBufferType
// InputBoxBufferType
// SelectorBufferType
#[derive(PartialEq, Eq)]
pub enum BufferLogic {
    Editor,
    InputBox,
    Selector,
}

pub struct Buffer {
    pub id: Uuid,
    pub is_overlay: bool,

    pub data: BufferData,

    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,

    pub scroll_x: usize,
    pub scroll_y: usize,

    pub file_path: Option<PathBuf>,
    read_only: bool,
    pub visible: bool,
    pub line_numbers: bool,

    pub logic: BufferLogic,

    msg_sender: Sender<EditorEvent>,
    paused_event_id: Uuid,
}

impl Buffer {
    pub fn new(
        path: PathBuf,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        logic: BufferLogic,
        msg_sender: Sender<EditorEvent>,
    ) -> io::Result<Self> {
        if path.exists() && !path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The path is neither a file nor a directory",
            ));
        }

        // Create an empty BufferData or populate it from the file
        let data = if path.is_file() {
            // If it's a file, read the file into a Vec<u8> and create a BufferData from it
            let contents = fs::read_to_string(&path)?;
            // Use the `BufferData::from` function to convert raw data into `BufferData`
            BufferData::from(contents)
        } else {
            // If it's a directory, we can't load it into a buffer, so we initialize an empty BufferData
            BufferData::new()
        };

        let line_numbers = match logic {
            BufferLogic::Editor => true,
            BufferLogic::InputBox => false,
            BufferLogic::Selector => false,
        };

        // Create a new Buffer instance
        Ok(Self {
            id: Uuid::nil(),       // nil UUID
            is_overlay: false,     // Default to not overlaying
            data,                  // Set the BufferData
            x,                     // Position x
            y,                     // Position y
            width,                 // Width
            height,                // Height
            scroll_x: 0,           // Default scroll position
            scroll_y: 0,           // Default scroll position
            file_path: Some(path), // Store the file path
            read_only: false,      // Default to not read-only
            visible: true,         // Default to visible
            line_numbers,
            logic,                 // Default logic type is Editor
            msg_sender,            // Channel to send messages to editor
            paused_event_id: Uuid::nil(),
        })
    }

    pub fn set_paused_event_id(&mut self, id: Uuid) {
        self.paused_event_id = id;
    }

    pub fn set_path(&mut self, path: PathBuf) -> Result<(), ()> {
        if path.file_name().is_none() || (path.exists() && path.is_dir()) {
            return Err(());
        } else if path.exists() && !path.is_file() {
            return Err(());
        }

        self.file_path = Some(path);
        Ok(())
    }

    pub fn move_to(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.width = w;
        self.height = h;
    }

    /// Returns the cursor x, y position on Terminal
    /// Position can be negative, which usually means cursor is currently outside the displayable bounds
    #[allow(unused_assignments)]
    pub fn cursor_xy(&self) -> (isize, isize) {
        let mut x = 0isize;
        let mut y = 0isize;

        let line_numbers_offset = if self.line_numbers {
            self.data.digits_in_line_num() as isize
        } else {
            0
        };

        for Line { start, end } in self.data.lines.iter() {
            if *start <= self.data.cursor && *end >= self.data.cursor {
                x = self.data.cursor as isize - *start as isize - self.scroll_x as isize + line_numbers_offset;

                return (
                    x + self.x as isize,
                    y - self.scroll_y as isize + self.y as isize,
                );
            } else {
                y += 1;
            }
        }

        let last_line = self
            .data
            .lines
            .last()
            .expect("Buffer should always have atleast one line");

        (
            last_line.end as isize - last_line.start as isize + 1 + self.x as isize
                - self.scroll_x as isize + line_numbers_offset,
            y - 1 - self.scroll_y as isize + self.y as isize,
        )
    }

    /// Returns the x, y position of the cursor relative to current buffer only
    #[allow(unused_assignments)]
    pub fn cursor_xy_relative(&self) -> (usize, usize) {
        let mut x = 0usize;
        let mut y = 0usize;

        for Line { start, end } in self.data.lines.iter() {
            if *start <= self.data.cursor && *end >= self.data.cursor {
                x = self.data.cursor - *start;

                return (x, y);
            } else {
                y += 1;
            }
        }

        let last_line = self
            .data
            .lines
            .last()
            .expect("Buffer should always have atleast one line");

        (last_line.end - last_line.start + 1, y - 1)
    }

    pub fn scroll(&mut self) {
        let (x, y) = self.cursor_xy();
        let (w, h) = (self.width, self.height);

        let line_numbers_offset = if self.line_numbers {
            self.data.digits_in_line_num() as isize
        } else {
            0
        };

        let y = y - self.y as isize;
        let x = x - self.x as isize - line_numbers_offset;

        if y < 0 {
            let dy = (-y) as usize;
            assert!(self.scroll_y >= dy);
            self.scroll_y -= dy; // NOTE: This could lead to overflow
        } else if y >= h as isize {
            let dy = y - h as isize + 1;
            self.scroll_y += dy as usize;
        }

        if x < 0 {
            let dx = (-x) as usize;
            self.scroll_x -= dx;
        } else if x >= w as isize {
            let dx = x - w as isize + 1;
            self.scroll_x += dx as usize;
        }
    }

    pub fn parse_input(&mut self, event: Event) {
        match self.logic {
            BufferLogic::Editor => self.editor_logic(event),
            BufferLogic::InputBox => self.input_box_logic(event),
            BufferLogic::Selector => todo!(),
        }
    }

    pub fn editor_logic(&mut self, event: Event) {
        if self.read_only {
            return;
        }

        match event {
            // Save As
            Event::Key(KeyEvent {
                code: KeyCode::Char('S'),
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) if modifiers == KeyModifiers::CONTROL | KeyModifiers::SHIFT => {
                self.msg_sender
                    .send(EditorEvent::Buffer(BufferEvent::SaveAs { id: self.id }))
                    .expect("Failed to send a msg to the editor");
            }
            // Save
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.msg_sender
                    .send(EditorEvent::Buffer(BufferEvent::Save { id: self.id }))
                    .expect("Failed to send a msg to the editor");
            }

            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.move_cursor_left(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.move_cursor_right(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.move_cursor_up(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.move_cursor_down(1);
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.insert_ch(c);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.insert_ch(c.to_ascii_uppercase());
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.insert_ch('\n');
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.backspace();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.delete();
            }

            _ => (),
        }

        self.data.recalculate_lines();
        self.scroll();
    }

    pub fn input_box_logic(&mut self, event: Event) {
        if self.read_only {
            return;
        }

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.move_cursor_left(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.move_cursor_right(1);
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.insert_ch(c);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.insert_ch(c.to_ascii_uppercase());
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let result: String = self.data.data.iter().collect();
                self.msg_sender
                    .send(EditorEvent::Buffer(BufferEvent::ResumeEvent {
                        paused_event_id: self.paused_event_id,
                        result,
                    }))
                    .unwrap();

                self.msg_sender
                    .send(EditorEvent::Buffer(BufferEvent::Close {
                        id: self.id,
                        is_overlay: self.is_overlay,
                    }))
                    .unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.msg_sender
                    .send(EditorEvent::Buffer(BufferEvent::CancelEvent {
                        paused_event_id: self.paused_event_id,
                    }))
                    .unwrap();

                self.msg_sender
                    .send(EditorEvent::Buffer(BufferEvent::Close {
                        id: self.id,
                        is_overlay: self.is_overlay,
                    }))
                    .unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.backspace();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.data.delete();
            }

            _ => (),
        }

        self.data.recalculate_lines();
        self.scroll();
    }
}
