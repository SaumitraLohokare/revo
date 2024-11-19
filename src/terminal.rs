#![allow(dead_code)]
use std::{
    io::{self, Write},
    process::exit,
};

use crossterm::{
    cursor::{Hide, MoveTo, SetCursorStyle, Show},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, DisableLineWrap, EnableLineWrap,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use crate::{
    buffer::{Buffer, BufferLogic, Padding},
    status_line::StatusLine,
    theme::Theme,
    vec_ext::VecExt,
};

enum BrushEvent {
    SetBG(Color),
    PreviousBG,
    SetFG(Color),
    PreviousFG,
}

pub struct Terminal<W: Write> {
    pub width: u16,
    pub height: u16,
    buffer: Vec<Vec<char>>,
    brushes: Vec<Vec<(usize, BrushEvent)>>, // TODO: Look into a better way for this
    out: W,
}

impl<W: Write> Terminal<W> {
    pub fn new(out: W) -> io::Result<Self> {
        let size = terminal::size()?;

        let buffer = (0..size.1)
            .into_iter()
            .map(|_| (0..size.0).map(|_| ' ').collect())
            .collect();

        let brushes = (0..size.1).into_iter().map(|_| vec![]).collect();

        enable_raw_mode()?;

        let mut display = Self {
            width: size.0,
            height: size.1,
            buffer,
            brushes,
            out,
        };

        execute!(display.out, EnterAlternateScreen, DisableLineWrap)?;

        Ok(display)
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.width = w;
        self.height = h;

        self.buffer = (0..h)
            .into_iter()
            .map(|_| (0..w).map(|_| ' ').collect())
            .collect();

        self.brushes = (0..h).into_iter().map(|_| vec![]).collect();
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }

    pub fn begin_draw(&mut self, theme: &Theme) -> io::Result<()> {
        for row in self.buffer.iter_mut() {
            row.clear();
            row.fill_to_capacity(' ');
        }

        for brush in self.brushes.iter_mut() {
            brush.clear();
            brush.push((0, BrushEvent::SetBG(Theme::hex_to_color(&theme.ui.base_bg))));
            brush.push((
                0,
                BrushEvent::SetFG(Theme::hex_to_color(&theme.ui.base_text)),
            ));
        }

        queue!(self.out, ResetColor, Hide)
    }

    pub fn end_draw(&mut self) -> io::Result<()> {
        for (i, row) in self.buffer.iter().enumerate() {
            let row_burshes = &mut self.brushes[i];

            row_burshes.sort_by_key(|b| b.0); // Sort based on colors

            let mut start_idx = 0;
            let mut bg_prev_color = None;
            let mut bg_prev_prev_color = None; // Hack... I wish there is a better way
            let mut fg_prev_color = None;
            let mut fg_prev_prev_color = None; // Hack... I wish there is a better way
            for (idx, color) in row_burshes {
                let colored_str = &row[start_idx..*idx];
                let colored_str: String = colored_str.into_iter().collect();
                queue!(
                    self.out,
                    MoveTo(start_idx as u16, i as u16),
                    Print(colored_str)
                )?;
                match color {
                    BrushEvent::SetBG(color) => {
                        queue!(self.out, SetBackgroundColor(*color))?;
                        bg_prev_prev_color = bg_prev_color;
                        bg_prev_color = Some(*color);
                    }
                    BrushEvent::PreviousBG => {
                        let bg_color = bg_prev_prev_color
                            .expect("First brush event should never be PreviousBG");
                        queue!(self.out, SetBackgroundColor(bg_color))?;
                        bg_prev_prev_color = bg_prev_color;
                        bg_prev_color = Some(bg_color);
                    }
                    BrushEvent::SetFG(color) => {
                        queue!(self.out, SetForegroundColor(*color))?;
                        fg_prev_prev_color = fg_prev_color;
                        fg_prev_color = Some(*color);
                    }
                    BrushEvent::PreviousFG => {
                        let fg_color = fg_prev_prev_color
                            .expect("First brush event should never be PreviousBG");
                        queue!(self.out, SetForegroundColor(fg_color))?;
                        fg_prev_prev_color = fg_prev_color;
                        fg_prev_color = Some(fg_color);
                    }
                };
                start_idx = *idx;
            }
            let colored_str = &row[start_idx..];
            let colored_str: String = colored_str.into_iter().collect();
            queue!(
                self.out,
                MoveTo(start_idx as u16, i as u16),
                Print(colored_str)
            )?;
        }

        self.flush()
    }

    /// Paint's the terminal row background, from start column to end column (exclusive) with color.
    ///
    /// Color is provided as Hex RGB (#FFFFFF)
    fn paint_bg(&mut self, row: usize, start: usize, end: usize, color: &str) {
        self.brushes[row].push((start, BrushEvent::SetBG(Theme::hex_to_color(color))));

        self.brushes[row].push((end, BrushEvent::PreviousBG));
    }

    /// Paint's the terminal row foreground, from start column to end column (exclusive) with color.
    ///
    /// Color is provided as Hex RGB (#FFFFFF)
    fn paint_fg(&mut self, row: usize, start: usize, end: usize, color: &str) {
        self.brushes[row].push((start, BrushEvent::SetFG(Theme::hex_to_color(color))));

        self.brushes[row].push((end, BrushEvent::PreviousFG));
    }

    pub fn clear(&mut self) -> io::Result<()> {
        let line: String = (0..self.width).into_iter().map(|_| ' ').collect();
        for i in 0..self.height {
            queue!(self.out, MoveTo(0, i), Print(&line))?;
        }

        Ok(())
    }

    pub fn set_cursor_style(&mut self, style: SetCursorStyle) -> io::Result<()> {
        queue!(self.out, style)
    }

    pub fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        queue!(self.out, MoveTo(x, y))
    }

    pub fn print(&mut self, string: String) -> io::Result<()> {
        queue!(self.out, Print(string))
    }

    pub fn draw_buffer(&mut self, buffer: &Buffer, theme: &Theme) {
        let Padding {
            top, bottom, left, ..
        } = buffer.padding();

        let mut row_idx = buffer.y as usize + top;
        let buf_x = buffer.x as usize;
        let buf_end = (buffer.x + buffer.width) as usize;
        let start_x = buf_x + left;
        let buf_current_line = buffer.data.current_line();

        let height = std::cmp::min(buffer.height, self.height);

        let border_bg_color = match buffer.is_overlay {
            true => &theme.overlay.bg,
            false => &theme.editor.bg,
        };
        let border_fg_color = match buffer.is_overlay {
            true => &theme.overlay.text,
            false => &theme.editor.text,
        };

        if buffer.bordered {
            self.buffer[buffer.y as usize].replace_from(buf_x, &buffer.top_border);

            self.paint_bg(buffer.y as usize, buf_x, buf_end, border_bg_color);
            self.paint_fg(buffer.y as usize, buf_x, buf_end, border_fg_color);
        }

        for line_num in (buffer.scroll_y..buffer.data.line_count())
            .take((height as usize).saturating_sub(bottom + top))
        {
            if let Some(display_line) = buffer.get_row(line_num) {
                self.buffer[row_idx].replace_from(buf_x, &display_line);

                match buffer.logic {
                    BufferLogic::Editor => {
                        let line_color = if buf_current_line == line_num {
                            &theme.editor.current_line
                        } else {
                            &theme.editor.bg
                        };

                        self.paint_bg(row_idx, buf_x, buf_end, &line_color);

                        if buffer.line_numbers {
                            let border_gap = if buffer.bordered { 1 } else { 0 };
                            self.paint_fg(
                                row_idx,
                                buf_x + border_gap,
                                start_x,
                                &theme.editor.line_numbers,
                            );
                        }

                        self.paint_fg(row_idx, start_x, buf_end, &theme.editor.text);
                    }
                    BufferLogic::InputBox => {
                        self.paint_bg(row_idx, buf_x, buf_end, &theme.overlay.bg);
                        self.paint_fg(row_idx, start_x, buf_end, &theme.overlay.text);
                    }
                    BufferLogic::Selector => todo!(),
                }
            }

            row_idx += 1;
        }

        if buffer.bordered {
            self.buffer[row_idx].replace_from(buf_x, &buffer.bottom_border);

            self.paint_bg(row_idx, buf_x, buf_end, border_bg_color);
            self.paint_fg(row_idx, buf_x, buf_end, border_fg_color);
        }
    }

    pub fn draw_status_line(&mut self, sl: &StatusLine, theme: &Theme) {
        let line = sl.get_line(self.width);

        self.buffer[self.height as usize - 1].replace_from(0, &line);
        self.brushes[self.height as usize - 1].push((
            0,
            BrushEvent::SetBG(Theme::hex_to_color(&theme.status_line.bg)),
        ));
        self.brushes[self.height as usize - 1].push((
            0,
            BrushEvent::SetFG(Theme::hex_to_color(&theme.status_line.text)),
        ));
        self.brushes[self.height as usize - 1].push((self.width as usize, BrushEvent::PreviousBG));
        self.brushes[self.height as usize - 1].push((self.width as usize, BrushEvent::PreviousFG));
    }

    pub fn draw_welcome_msg(&mut self) {
        let msg = vec!["Revo v0.1", "", "Quit: Ctrl + Q"];

        let x_center = (self.width as f32 * 0.5) as usize;
        let y_center = (self.height as f32 * 0.4) as usize;

        for (i, line) in msg.iter().enumerate() {
            let line_x = x_center - (line.len() / 2);
            let line_y = (y_center as i16 + (i as i16 - msg.len() as i16 / 2)) as usize;

            self.buffer[line_y].replace_from(line_x as usize, line);
        }
    }

    pub fn show_cursor(&mut self, buffer: &Buffer) -> io::Result<()> {
        let (cursor_x, cursor_y) = buffer.cursor_xy();

        if cursor_x >= buffer.x as isize
            && cursor_x < buffer.x as isize + buffer.width as isize
            && cursor_y >= buffer.y as isize
            && cursor_y < buffer.y as isize + buffer.height as isize
        {
            execute!(self.out, MoveTo(cursor_x as u16, cursor_y as u16), Show,)?;
        }

        Ok(())
    }
}

impl<W: Write> Drop for Terminal<W> {
    fn drop(&mut self) {
        if let Err(e) = disable_raw_mode() {
            eprintln!("ERROR : Failed to disable terminal raw mode : {e}");
            exit(1);
        }

        if let Err(e) = execute!(
            self.out,
            ResetColor,
            LeaveAlternateScreen,
            EnableLineWrap,
            SetCursorStyle::BlinkingBlock,
            Show
        ) {
            eprintln!("ERROR : Failed to leave alternate screen : {e}");
            exit(1);
        }
    }
}
