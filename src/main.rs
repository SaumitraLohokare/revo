use std::{
    io::{self, Write, stdout}, mem::swap, process::exit
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Color, PrintStyledContent, StyledContent, Stylize},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, DisableLineWrap, EnableLineWrap,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
    tty::IsTty,
};

fn main() -> io::Result<()> {
    // TODO: Set-up ignoring Ctrl+C

    // Terminal Setup -----------

    let mut out = stdout();
    if !out.is_tty() {
        eprintln!("ERROR: Please run this in a terminal");
        exit(1);
    }

    enable_raw_mode()?;
    execute!(out, EnterAlternateScreen, DisableLineWrap, Hide)?;

    // Bufer Setup --------------
    let mut insert_mode = false;
    let mut buff = Buffer::empty();

    // Input Loop ---------------
    let (w, h) = terminal::size()?;
    let mut view = View::new(w, h);
    render_cells(&out, view.make_cells(&buff))?;

    while let Ok(evt) = read() {
        use KeyCode::*;

        match evt {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match (code, modifiers) {
                (Char('q'), KeyModifiers::CONTROL) => break,
                (Char('i'), KeyModifiers::NONE) if !insert_mode => {
                    buff.clear_selection();
                    insert_mode = true;
                }
                (Char('a'), KeyModifiers::NONE) if !insert_mode => {
                    buff.clear_selection();
                    buff.move_cursor_right(false);
                    insert_mode = true;
                }
                (Char('I'), KeyModifiers::SHIFT) if !insert_mode => {
                    buff.clear_selection();
                    buff.move_cursor_to_line_start(false);
                    insert_mode = true;
                }
                (Char('A'), KeyModifiers::SHIFT) if !insert_mode => {
                    buff.clear_selection();
                    buff.move_cursor_to_line_end(false);
                    insert_mode = true;
                }
                (Esc, KeyModifiers::NONE) if insert_mode => {
                    insert_mode = false;
                }

                (Char('k'), KeyModifiers::NONE) if !insert_mode => buff.move_cursor_up(false),
                (Char('j'), KeyModifiers::NONE) if !insert_mode => buff.move_cursor_down(false),
                (Char('h'), KeyModifiers::NONE) if !insert_mode => buff.move_cursor_left(false),
                (Char('l'), KeyModifiers::NONE) if !insert_mode => buff.move_cursor_right(false),

                (Char('K'), KeyModifiers::SHIFT) if !insert_mode => buff.move_cursor_up(true),
                (Char('J'), KeyModifiers::SHIFT) if !insert_mode => buff.move_cursor_down(true),
                (Char('H'), KeyModifiers::SHIFT) if !insert_mode => buff.move_cursor_left(true),
                (Char('L'), KeyModifiers::SHIFT) if !insert_mode => buff.move_cursor_right(true),

                (Char('b'), KeyModifiers::NONE) if !insert_mode => {
                    buff.move_cursor_to_word_begining(false)
                }
                (Char('w'), KeyModifiers::NONE) if !insert_mode => {
                    buff.move_cursor_to_next_word_begining(false)
                }
                (Char('e'), KeyModifiers::NONE) if !insert_mode => {
                    buff.move_cursor_to_word_end(false)
                }
                (Char('.'), KeyModifiers::NONE) if !insert_mode => {
                    buff.move_cursor_to_line_end(false)
                }
                (Char(','), KeyModifiers::NONE) if !insert_mode => {
                    buff.move_cursor_to_line_start(false)
                }

                (Char('B'), KeyModifiers::SHIFT) if !insert_mode => {
                    // TODO: Back selection should not include the character itself
                    buff.move_cursor_to_word_begining(true)
                }
                (Char('W'), KeyModifiers::SHIFT) if !insert_mode => {
                    buff.move_cursor_to_next_word_begining(true)
                }
                (Char('E'), KeyModifiers::SHIFT) if !insert_mode => {
                    buff.move_cursor_to_word_end(true)
                }
                (Char('>'), KeyModifiers::SHIFT) if !insert_mode => {
                    buff.move_cursor_to_line_end(true)
                }
                (Char('<'), KeyModifiers::SHIFT) if !insert_mode => {
                    buff.move_cursor_to_line_start(true)
                }

                (Char('o'), KeyModifiers::NONE) if !insert_mode => {
                    buff.insert_line_below();
                    buff.move_cursor_down(false);
                    insert_mode = true;
                }
                (Char('O'), KeyModifiers::SHIFT) if !insert_mode => {
                    buff.insert_line_at_cursor();
                    insert_mode = true;
                }

                // Regardless of mode:
                (Char('j'), KeyModifiers::CONTROL) => buff.move_line_down(),
                (Char('k'), KeyModifiers::CONTROL) => buff.move_line_up(),
                (Char('j'), KeyModifiers::ALT) => {
                    let mut line = buff.cursor_line().clone();
                    buff.insert_line_below();
                    buff.cursor_line_n_mut(1).append(&mut line);
                    buff.move_cursor_down(false);
                }
                (Char('k'), KeyModifiers::ALT) => {
                    let x = buff.cursor_x; // remember old x pos
                    let mut line = buff.cursor_line().clone();
                    buff.insert_line_at_cursor();
                    buff.cursor_line_mut().append(&mut line);
                    buff.cursor_x = x; // retain old x
                }

                (Char(ch), KeyModifiers::NONE) if insert_mode => {
                    buff.insert_char_at_cursor(ch);
                    buff.move_cursor_right(false);
                }
                (Char(ch), KeyModifiers::SHIFT) if insert_mode => {
                    buff.insert_char_at_cursor(ch);
                    buff.move_cursor_right(false);
                }
                (Enter, KeyModifiers::NONE) if insert_mode => {
                    let x = buff.cursor_x;
                    let line = buff.cursor_line_mut();
                    let mut right = line.split_off(x);
                    buff.insert_line_below();
                    buff.move_cursor_right(false);
                    swap(buff.cursor_line_mut(), &mut right);
                }
                (Backspace, KeyModifiers::NONE) if insert_mode => {
                    if buff.cursor_x > 0 {
                        buff.move_cursor_left(false);
                        buff.delete_char_at_cursor();
                    } else {
                        let mut line = buff.remove_cursor_line();
                        buff.move_cursor_left(false);
                        buff.cursor_line_mut().append(&mut line);
                    }
                }
                _ => {}
            },
            Event::Resize(w, h) => view = View::new(w, h),
            _ => {}
        }

        // Render ------------

        render_cells(&out, view.make_cells(&buff))?;
    }

    // Terminal Exit ----------------

    execute!(out, LeaveAlternateScreen, EnableLineWrap, Show)?;
    disable_raw_mode()?;

    Ok(())
}

struct SelectionAnchor {
    x: usize,
    y: usize,
}

struct Buffer {
    cursor_x: usize,
    cursor_y: usize,
    selection: Option<SelectionAnchor>,
    lines: Vec<Vec<char>>,
}

impl Buffer {
    fn empty() -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            selection: None,
            lines: vec![vec![]],
        }
    }

    #[inline]
    fn clear_selection(&mut self) {
        self.selection = None;
    }

    fn handle_selection(&mut self, extend_selection: bool) {
        if !extend_selection && self.selection.is_some() {
            self.clear_selection();
        } else if extend_selection && self.selection.is_none() {
            self.selection = Some(SelectionAnchor {
                x: self.cursor_x,
                y: self.cursor_y,
            })
        }
    }

    fn is_cursor(&self, x: usize, y: usize) -> bool {
        self.cursor_x == x && self.cursor_y == y
    }

    fn is_in_selection(&self, x: usize, y: usize, w: usize) -> bool {
        let Some(SelectionAnchor { x: ax, y: ay }) = self.selection else {
            return false;
        };

        let anchor = ay * w + ax;
        let cursor = self.cursor_y * w + self.cursor_x;
        let pos = y * w + x;

        (pos >= anchor && pos < cursor) || (pos >= cursor && pos < anchor)
    }

    fn cursor_line(&self) -> &Vec<char> {
        self.lines
            .get(self.cursor_y)
            .expect("Buffer::cursor_line failed")
    }

    fn cursor_line_mut(&mut self) -> &mut Vec<char> {
        self.lines
            .get_mut(self.cursor_y)
            .expect("Buffer::cursor_line failed")
    }
    
    fn cursor_line_n_mut(&mut self, n: usize) -> &mut Vec<char> {
        assert!(self.cursor_y + n < self.lines.len());
        self.lines
            .get_mut(self.cursor_y + n)
            .expect("Buffer::cursor_line failed")
    }

    fn remove_cursor_line(&mut self) -> Vec<char> {
        self.lines.remove(self.cursor_y)
    }

    fn move_cursor_up(&mut self, extend_selection: bool) {
        self.handle_selection(extend_selection);

        if self.cursor_y > 0 {
            self.cursor_y -= 1;
        }
        let line_len = self.cursor_line().len();
        if self.cursor_x > line_len {
            self.cursor_x = line_len;
        }
    }

    fn move_cursor_down(&mut self, extend_selection: bool) {
        self.handle_selection(extend_selection);

        let new_y = self.cursor_y + 1;
        if new_y < self.lines.len() {
            self.cursor_y = new_y;
        }
        let line_len = self.cursor_line().len();
        if self.cursor_x > line_len {
            self.cursor_x = line_len;
        }
    }

    fn move_cursor_left(&mut self, extend_selection: bool) {
        self.handle_selection(extend_selection);

        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            let line_len = self.cursor_line().len();
            self.cursor_x = line_len;
        }
    }

    fn move_cursor_right(&mut self, extend_selection: bool) {
        self.handle_selection(extend_selection);

        let new_x = self.cursor_x + 1;
        let line_len = self.lines.get(self.cursor_y).unwrap().len();
        if new_x <= line_len {
            self.cursor_x = new_x;
        } else if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    fn move_cursor_to_line_end(&mut self, extend_selection: bool) {
        self.handle_selection(extend_selection);

        let line = self.cursor_line();
        let (idx, _) = line
            .iter()
            .enumerate()
            .rev()
            .filter(|(idx, _)| *idx > self.cursor_x)
            .skip_while(|(_, ch)| ch.is_whitespace())
            .next()
            .unwrap_or((line.len(), &' '));
        self.cursor_x = if idx + 1 > line.len() {
            line.len()
        } else {
            idx + 1 // To skip to the next whitespace
        };
    }

    fn move_cursor_to_line_start(&mut self, extend_selection: bool) {
        self.handle_selection(extend_selection);

        let line = self.cursor_line();
        let (idx, _) = line
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx < self.cursor_x)
            .skip_while(|(_, ch)| ch.is_whitespace())
            .next()
            .unwrap_or((0, &' '));
        self.cursor_x = idx;
    }

    fn move_cursor_to_word_begining(&mut self, extend_selection: bool) {
        self.handle_selection(extend_selection);

        // check in our line for previous word begining
        // if failed, move cursor to line begining
        if self.cursor_x > 0 {
            let line = self.cursor_line();
            self.cursor_x = self.find_previous_word_begining(line, self.cursor_x - 1);
        } else if self.cursor_y > 0 {
            // if cursor at line begining check from previous line
            self.cursor_y -= 1;
            let line = self.cursor_line();
            self.cursor_x = self.find_previous_word_begining(line, line.len().saturating_sub(1));
        }
    }

    fn find_previous_word_begining(&self, line: &[char], mut idx: usize) -> usize {
        let mut found_non_delimiter = false;
        loop {
            let Some(ch) = line.get(idx) else { break };
            // TODO: This should change to delimiters
            if is_delimiter(*ch) && found_non_delimiter {
                idx += 1;
                break;
            } else if idx > 0 {
                idx -= 1;
                found_non_delimiter = found_non_delimiter || !is_delimiter(*ch);
            } else {
                break;
            }
        }
        idx
    }

    fn move_cursor_to_next_word_begining(&mut self, extend_selection: bool) {
        self.handle_selection(extend_selection);

        // check in our line for previous word begining
        // if failed, move cursor to line begining
        let line = self.cursor_line();
        if self.cursor_x < line.len() {
            self.cursor_x = self.find_next_word_begining(line, self.cursor_x, false);
        } else if self.cursor_y < self.lines.len() - 1 {
            // if cursor at line begining check from previous line
            self.cursor_y += 1;
            let line = self.cursor_line();
            self.cursor_x = self.find_next_word_begining(line, 0, true);
        }
    }

    fn find_next_word_begining(&self, line: &[char], mut idx: usize, is_next_line: bool) -> usize {
        let mut found_delimiter = is_next_line;
        loop {
            let Some(ch) = line.get(idx) else { break };
            // TODO: This should change to delimiters
            if !is_delimiter(*ch) && found_delimiter {
                break;
            } else if idx < line.len() {
                idx += 1;
                found_delimiter = found_delimiter || is_delimiter(*ch);
            } else {
                break;
            }
        }
        idx
    }

    fn move_cursor_to_word_end(&mut self, extend_selection: bool) {
        self.handle_selection(extend_selection);

        // check in our line for previous word begining
        // if failed, move cursor to line begining
        let line = self.cursor_line();
        if self.cursor_x < line.len() {
            self.cursor_x = self.find_word_end(line, self.cursor_x);
        } else if self.cursor_y < self.lines.len() - 1 {
            // if cursor at line begining check from previous line
            self.cursor_y += 1;
            let line = self.cursor_line();
            self.cursor_x = self.find_word_end(line, 0);
        }
    }

    fn find_word_end(&self, line: &[char], mut idx: usize) -> usize {
        let mut found_non_delimiter = false;
        loop {
            let Some(ch) = line.get(idx) else { break };
            // TODO: This should change to delimiters
            if is_delimiter(*ch) && found_non_delimiter {
                break;
            } else if idx < line.len() {
                idx += 1;
                found_non_delimiter = found_non_delimiter || !is_delimiter(*ch);
            } else {
                break;
            }
        }
        idx
    }

    fn move_line_down(&mut self) {
        let y = self.cursor_y;
        if y + 1 < self.lines.len() {
            self.lines.swap(y, y + 1);
            self.cursor_y += 1;
        }
    }

    fn move_line_up(&mut self) {
        let y = self.cursor_y;
        if y > 0 {
            self.lines.swap(y, y - 1);
            self.cursor_y -= 1;
        }
    }

    fn insert_char_at_cursor(&mut self, ch: char) {
        self.lines
            .get_mut(self.cursor_y)
            .unwrap()
            .insert(self.cursor_x, ch);
    }

    fn insert_line_below(&mut self) {
        self.lines.insert(self.cursor_y + 1, Vec::new());
    }

    fn insert_line_at_cursor(&mut self) {
        self.lines.insert(self.cursor_y, Vec::new());
        self.cursor_x = 0;
    }

    fn delete_char_at_cursor(&mut self) {
        let x = self.cursor_x;
        let line = self.cursor_line_mut();
        if x < line.len() {
            line.remove(x);
        }
    }
}

fn is_delimiter(ch: char) -> bool {
    ch.is_whitespace()
}

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    fg: Color,
    bg: Color,
}

impl Cell {
    fn empty() -> Self {
        Self {
            ch: ' ',
            fg: Color::Rgb {
                r: 234,
                g: 234,
                b: 234,
            },
            bg: Color::Rgb {
                r: 31,
                g: 35,
                b: 41,
            },
        }
    }

    fn char(ch: char) -> Self {
        Self {
            ch,
            fg: Color::Rgb {
                r: 234,
                g: 234,
                b: 234,
            },
            bg: Color::Rgb {
                r: 31,
                g: 35,
                b: 41,
            },
        }
    }

    fn styled(&self) -> StyledContent<char> {
        self.ch.on(self.bg).with(self.fg)
    }

    fn set_color(&mut self, bg: Color, fg: Color) {
        self.bg = bg;
        self.fg = fg;
    }
}

type Cells = Vec<Vec<Cell>>;

struct View {
    w: u16,
    h: u16,
}

impl View {
    fn new(w: u16, h: u16) -> Self {
        Self { w, h }
    }

    fn make_cells(&self, buff: &Buffer) -> Cells {
        let w = self.w as usize;
        let h = self.h as usize;

        let mut cells = Vec::new();

        let bg = Color::Rgb {
            r: 31,
            g: 35,
            b: 41,
        };

        let fg = Color::Rgb {
            r: 234,
            g: 234,
            b: 234,
        };

        let sel_bg = Color::Rgb {
            r: 51,
            g: 59,
            b: 71,
        };

        for y in 0..h {
            let mut row = Vec::new();
            for x in 0..w {
                let mut cell = match buff.lines.get(y).map(|line| line.get(x)) {
                    Some(Some(ch)) => {
                        let mut cell = Cell::char(*ch);

                        // Selection should only be colored for visible chars
                        if buff.is_in_selection(x, y, w) {
                            cell.set_color(sel_bg, fg);
                        }

                        cell
                    }
                    _ => Cell::empty(),
                };

                // Cursor can be outside visible chars
                if buff.is_cursor(x, y) {
                    cell.set_color(fg, bg);
                }

                row.push(cell);
            }
            cells.push(row);
        }

        cells
    }
}

fn render_cells<W: Write>(mut out: W, cells: Cells) -> io::Result<()> {
    for (x_idx, row) in cells.iter().enumerate() {
        queue!(out, MoveTo(0, x_idx as u16))?;
        for cell in row {
            queue!(out, PrintStyledContent(cell.styled()))?;
        }
    }
    out.flush()
}
