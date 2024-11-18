use std::char;

pub trait VecExt {
    fn fill_to_capacity(&mut self, ch: char);
    fn replace_from(&mut self, index: usize, string: &str);
}

impl VecExt for Vec<char> {
    /// Fill's the Vec with given character till capacity
    fn fill_to_capacity(&mut self, ch: char) {
        for _ in self.len()..self.capacity() {
            self.push(ch);
        }
    }
    
    /// Replaces chars in the `Vec` starting from given index with `char`s from `&str`.
    /// 
    /// NOTE: If `self.len()` ends before all the characters from `string` are inserted, then we stop.
    fn replace_from(&mut self, index: usize, with: &str) {
        let mut chars = with.chars();
        for i in index..self.len() {
            match chars.next() {
                Some(ch) => self[i] = ch,
                None => break,
            }
        }
    }
}