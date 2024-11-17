use std::char;

pub trait VecExt {
    fn fill_to_capacity(&mut self, ch: char);
    fn insert_str_at(&mut self, index: usize, string: &str);
}

impl VecExt for Vec<char> {
    fn fill_to_capacity(&mut self, ch: char) {
        for _ in self.len()..self.capacity() {
            self.push(ch);
        }
    }
    
    fn insert_str_at(&mut self, index: usize, string: &str) {
        let mut chars = string.chars();
        for i in index..self.len() {
            match chars.next() {
                Some(ch) => self[i] = ch,
                None => break,
            }
        }
    }
}