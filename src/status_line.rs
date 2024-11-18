pub struct StatusLine {
    file_name: String,
    cursor_x: usize,
    cursor_y: usize,
}

impl StatusLine {
    pub fn new() -> Self {
        Self {
            file_name: "NO NAME".to_string(),
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn update_file_name(&mut self, name: String) {
        self.file_name = name;
    }

    pub fn update_cursor_pos(&mut self, cursor_pos: (usize, usize)) {
        // We show line numbers starting from 1 so cursor position should also be starting from 1
        self.cursor_x = cursor_pos.0 + 1;
        self.cursor_y = cursor_pos.1 + 1;
    }

    pub fn get_line(&self, width: u16) -> String {
        let mut line = String::with_capacity(width as usize);
        line.push(' ');
        
        let mut content_width = self.file_name.len();
        line.push_str(&self.file_name);

        let cursor_x_str = self.cursor_x.to_string();
        let cursor_y_str = self.cursor_y.to_string();
        // "(x, y)"
        content_width += 1 + cursor_x_str.len() + 2 + cursor_y_str.len() + 1;

        for _ in 0..(width as usize - 2 - content_width) {
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
}
