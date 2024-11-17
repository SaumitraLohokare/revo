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
    buffer::{Buffer, Line},
    status_line::StatusLine,
    string::StringExt,
    theme::Theme, vec_ext::VecExt,
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
    // buffer: Vec<String>,
    buffer: Vec<Vec<char>>,
    brushes: Vec<Vec<(usize, BrushEvent)>>,
    out: W,
}

impl<W: Write> Terminal<W> {
    pub fn new(out: W) -> io::Result<Self> {
        let size = terminal::size()?;

        let buffer = (0..size.1)
            .into_iter()
            .map(|_| {
                (0..size.0).map(|_| ' ').collect()
            })
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
            .map(|_| {
                (0..w).map(|_| ' ').collect()
            })
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
            brush.push((0, BrushEvent::SetFG(Theme::hex_to_color(&theme.ui.base_text))));
        }

        queue!(self.out, ResetColor, Hide)
    }

    pub fn end_draw(&mut self) -> io::Result<()> {
        for (i, row) in self.buffer.iter().enumerate() {
            let row_burshes = &mut self.brushes[i];

            // if row_burshes.len() == 0 {
            //     queue!(self.out, MoveTo(0, i as u16), Print(row))?;
            // }

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
                        let bg_color = bg_prev_prev_color.expect("First brush event should never be PreviousBG");
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
                        let fg_color = fg_prev_prev_color.expect("First brush event should never be PreviousBG");
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
        let mut row_idx = buffer.y;
        let buf_current_line = buffer.data.current_line();

        let height = std::cmp::min(buffer.height, self.height);

        for (line_num, Line { start, end }) in buffer
            .data
            .lines
            .iter()
            .enumerate()
            .skip(buffer.scroll_y)
            .take(height as usize)
        {
            let mut display_line = String::with_capacity(buffer.width as usize);
            if let Some(data) = buffer.data.data.get(*start..=*end) {
                let line: String = data
                    .iter()
                    .skip(buffer.scroll_x)
                    .take(buffer.width as usize)
                    .filter(|c| **c != '\n')
                    .collect();
                display_line.push_str(&line);
            }
            display_line.fill_to_capacity(' ');
            self.buffer[row_idx as usize].insert_str_at(buffer.x as usize, &display_line);

            match buffer.logic {
                crate::buffer::BufferLogic::Editor => {
                    if buf_current_line == line_num {
                        self.brushes[row_idx as usize]
                            .push((buffer.x as usize, BrushEvent::SetBG(Theme::hex_to_color(&theme.editor.current_line))));
                    } else {
                        self.brushes[row_idx as usize]
                            .push((buffer.x as usize, BrushEvent::SetBG(Theme::hex_to_color(&theme.editor.bg))));
                    }

                    self.brushes[row_idx as usize]
                        .push((buffer.x as usize, BrushEvent::SetFG(Theme::hex_to_color(&theme.editor.text))));

                    self.brushes[row_idx as usize]
                        .push(((buffer.x + buffer.width) as usize, BrushEvent::PreviousBG));

                    self.brushes[row_idx as usize]
                        .push(((buffer.x + buffer.width) as usize, BrushEvent::PreviousFG));
                }
                crate::buffer::BufferLogic::InputBox => {
                    self.brushes[row_idx as usize]
                        .push((buffer.x as usize, BrushEvent::SetBG(Theme::hex_to_color(&theme.overlay.bg))));

                    self.brushes[row_idx as usize]
                        .push((buffer.x as usize, BrushEvent::SetFG(Theme::hex_to_color(&theme.overlay.text))));

                    self.brushes[row_idx as usize]
                        .push(((buffer.x + buffer.width) as usize, BrushEvent::PreviousBG));

                    self.brushes[row_idx as usize]
                        .push(((buffer.x + buffer.width) as usize, BrushEvent::PreviousFG));
                }
                crate::buffer::BufferLogic::Selector => todo!(),
            }

            row_idx += 1;
        }
    }

    pub fn draw_status_line(&mut self, sl: &StatusLine, theme: &Theme) {
        let line = sl.get_line(self.width);

        self.buffer[self.height as usize - 1].insert_str_at(0, &line);
        self.brushes[self.height as usize - 1].push((0, BrushEvent::SetBG(Theme::hex_to_color(&theme.status_line.bg))));
        self.brushes[self.height as usize - 1].push((0, BrushEvent::SetFG(Theme::hex_to_color(&theme.status_line.text))));
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

            self.buffer[line_y].insert_str_at(line_x as usize, line);
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
