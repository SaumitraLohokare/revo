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
    data: Vec<char>,
    lines: Vec<Line>,
    cursor: usize,
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
            .filter(|c| *c != '\r') // TODO: Maybe we can store if the file uses CRLF, and make the changes before saving
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

    pub fn line_count(&self) -> usize {
        self.lines.len()
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

impl ToString for BufferData {
    fn to_string(&self) -> String {
        self.data.iter().collect()
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

pub struct Padding {
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub left: usize,
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
    pub show_status_line: bool,

    pub bordered: bool,
    pub top_border: String,
    pub bottom_border: String,

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
        show_status_line: bool,
        bordered: bool,
        logic: BufferLogic,
        title: &str,
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

        let top_border = if bordered {
            let mut s = String::from('╭');
            s.push_str(title);
            let border_dash_len = (width - 2) as usize - title.len();
            s.push_str(&"─".repeat(border_dash_len));
            s.push('╮');
            s
        } else {
            String::new()
        };

        let bottom_border = if bordered {
            let mut s = String::from('╰');
            let border_dash_len = (width - 2) as usize;
            s.push_str(&"─".repeat(border_dash_len));
            s.push('╯');
            s
        } else {
            String::new()
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
            show_status_line,
            bordered,
            top_border,
            bottom_border,
            logic,      // Default logic type is Editor
            msg_sender, // Channel to send messages to editor
            paused_event_id: Uuid::nil(),
        })
    }

    pub fn get_row(&self, row: usize) -> Option<String> {
        let line = if let Some(line) = self.data.lines.get(row) {
            line
        } else {
            return None;
        };

        let mut num_chars = 0;

        let digits_in_line_nums = if self.line_numbers {
            self.data.digits_in_line_num()
        } else {
            0
        };

        let Padding { right, left, .. } = self.get_padding();

        let mut display_line = String::with_capacity(self.width as usize);
        if self.bordered {
            display_line.push('│');
            num_chars += 1;
        }

        let chars_to_take = self.width as usize - left - right;

        if self.line_numbers {
            let digits_count = (row + 1).ilog10() + 1;
            let spaces = (digits_in_line_nums - 1).saturating_sub(digits_count as usize);
            display_line.push_str(&" ".repeat(spaces));
            num_chars += spaces;

            display_line.push_str(&(row + 1).to_string());
            num_chars += digits_count as usize;

            display_line.push(' '); // fill the gap at the end
            num_chars += 1;
        }

        if let Some(line_chars) = self.data.data.get(line.start..=line.end) {
            for ch in line_chars
                .iter()
                .skip(self.scroll_x)
                .take(chars_to_take)
                .filter(|c| **c != '\n')
            {
                display_line.push(*ch);
                num_chars += 1;
            }
        }

        let spaces_to_add = (self.width as usize).saturating_sub(num_chars);
        display_line.push_str(&" ".repeat(spaces_to_add));
        if self.bordered {
            display_line.pop();
            display_line.push('│');
        }

        Some(display_line)
    }

    pub fn get_status_line(&self) -> String {
        let mut line = String::with_capacity(self.width as usize);
        line.push(' ');

        let file_name = match &self.file_path {
            Some(path) => match path.file_name() {
                Some(name) => name.to_str().unwrap().to_string(),
                None => "NO NAME".to_string(),
            }
            None => "NO NAME".to_string(),
        };
        
        let mut content_width = file_name.len();
        line.push_str(&file_name);

        let (cursor_x, cursor_y) = self.cursor_xy_relative();

        // Cursor position is 1-indexed
        let cursor_x_str = (cursor_x + 1).to_string();
        let cursor_y_str = (cursor_y + 1).to_string();

        // "(x, y)"
        content_width += 1 + cursor_x_str.len() + 2 + cursor_y_str.len() + 1;

        for _ in 0..(self.width as usize - 2 - content_width) {
            line.push(' ');
        }

        line.push('(');
        line.push_str(&cursor_x_str);
        line.push_str(", ");
        line.push_str(&cursor_y_str);
        line.push(')');

        line.push(' ');
        line
    }

    pub fn get_padding(&self) -> Padding {
        let line_numbers_offset = if self.line_numbers {
            self.data.digits_in_line_num()
        } else {
            0
        };

        let border_offset = if self.bordered { 1 } else { 0 };

        let status_line_offset = if self.show_status_line { 1 } else { 0 };

        Padding {
            top: border_offset,
            right: border_offset,
            bottom: border_offset + status_line_offset,
            left: border_offset + line_numbers_offset,
        }
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

        let Padding { left, top, .. } = self.get_padding();

        for Line { start, end } in self.data.lines.iter() {
            if *start <= self.data.cursor && *end >= self.data.cursor {
                x = self.data.cursor as isize - *start as isize - self.scroll_x as isize
                    + left as isize;

                return (
                    x + self.x as isize,
                    y - self.scroll_y as isize + self.y as isize + top as isize,
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
                - self.scroll_x as isize
                + left as isize,
            y - 1 - self.scroll_y as isize + self.y as isize + top as isize,
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

        let Padding {
            top,
            right,
            bottom,
            left,
        } = self.get_padding();

        let left_bound = self.x as isize + left as isize;
        let right_bound = (self.x + self.width) as isize - right as isize;
        let top_bound = self.y as isize + top as isize;
        let bot_bound = (self.y + self.height) as isize - bottom as isize;

        if y < top_bound {
            let dy = top_bound.saturating_sub(y) as usize;
            self.scroll_y = self.scroll_y.saturating_sub(dy);
        } else if y >= bot_bound {
            let dy = y - bot_bound + 1;
            self.scroll_y += dy as usize;
        }

        if x < left_bound {
            let dx = left_bound.saturating_sub(x) as usize;
            self.scroll_x = self.scroll_x.saturating_sub(dx);
        } else if x >= right_bound {
            let dx = x - right_bound + 1;
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

        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            match (code, modifiers) {
                // Save As: Ctrl + Shift + S
                (KeyCode::Char('S'), modifier)
                    if modifier == KeyModifiers::CONTROL | KeyModifiers::SHIFT =>
                {
                    self.msg_sender
                        .send(EditorEvent::Buffer(BufferEvent::SaveAs { id: self.id }))
                        .expect("Failed to send a msg to the editor");
                }
                // Save: Ctrl + S
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                    self.msg_sender
                        .send(EditorEvent::Buffer(BufferEvent::Save { id: self.id }))
                        .expect("Failed to send a msg to the editor");
                }
                // Arrow Keys (No modifiers)
                (KeyCode::Left, KeyModifiers::NONE) => {
                    self.data.move_cursor_left(1);
                }
                (KeyCode::Right, KeyModifiers::NONE) => {
                    self.data.move_cursor_right(1);
                }
                (KeyCode::Up, KeyModifiers::NONE) => {
                    self.data.move_cursor_up(1);
                }
                (KeyCode::Down, KeyModifiers::NONE) => {
                    self.data.move_cursor_down(1);
                }
                // Character insertion without modifiers
                (KeyCode::Char(c), KeyModifiers::NONE) => {
                    self.data.insert_ch(c);
                }
                // Character insertion with SHIFT (uppercase)
                (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                    self.data.insert_ch(c.to_ascii_uppercase());
                }
                // Enter key
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    self.data.insert_ch('\n');
                }
                // Backspace key
                (KeyCode::Backspace, KeyModifiers::NONE) => {
                    self.data.backspace();
                }
                // Delete key
                (KeyCode::Delete, KeyModifiers::NONE) => {
                    self.data.delete();
                }
                // Catch-all for any other key combinations (could be used for logging or other actions)
                _ => (),
            }
        }

        self.data.recalculate_lines();
        self.scroll();
    }

    pub fn input_box_logic(&mut self, event: Event) {
        if self.read_only {
            return;
        }

        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            match (code, modifiers) {
                // Cursor movement (Left, Right, Up, Down) - No modifiers
                (KeyCode::Left, KeyModifiers::NONE) => {
                    self.data.move_cursor_left(1);
                }
                (KeyCode::Right, KeyModifiers::NONE) => {
                    self.data.move_cursor_right(1);
                }
                (KeyCode::Up, KeyModifiers::NONE) => {
                    self.data.move_cursor_up(1);
                }
                (KeyCode::Down, KeyModifiers::NONE) => {
                    self.data.move_cursor_down(1);
                }

                // Character insertion (normal and shifted for uppercase)
                (KeyCode::Char(c), KeyModifiers::NONE) => {
                    self.data.insert_ch(c);
                }
                (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                    self.data.insert_ch(c.to_ascii_uppercase());
                }

                // Enter key press
                (KeyCode::Enter, KeyModifiers::NONE) => {
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

                // Escape key press (similar to Enter in terms of closing the event)
                (KeyCode::Esc, KeyModifiers::NONE) => {
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

                // Backspace and Delete keys
                (KeyCode::Backspace, KeyModifiers::NONE) => {
                    self.data.backspace();
                }
                (KeyCode::Delete, KeyModifiers::NONE) => {
                    self.data.delete();
                }

                // Catch-all for other key events that don't match any of the cases
                _ => (),
            }
        }

        self.data.recalculate_lines();
        self.scroll();
    }
}
