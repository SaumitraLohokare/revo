#![allow(dead_code)]

pub trait StringExt {
    fn fill_to_capacity(&mut self, c: char);
}

impl StringExt for String {
    /// Fills the string from `length` to `capacity` with given character.
    fn fill_to_capacity(&mut self, c: char) {
        for _ in self.len()..self.capacity() {
            self.push(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_to_capacity() {
        // Test when the string is initially empty and the capacity is greater than the length
        let mut s = String::with_capacity(10);
        assert_eq!(s.len(), 0);
        assert_eq!(s.capacity(), 10);
        
        s.fill_to_capacity('a');
        
        // Now the string should have 'a' repeated from len() to capacity
        assert_eq!(s.len(), 10);
        assert_eq!(s.capacity(), 10);
        assert_eq!(s, "aaaaaaaaaa");
        
        // Test when the string is initially at its full capacity
        let mut s = String::with_capacity(10);
        s.push('b');
        s.push('b');
        s.push('b');
        
        assert_eq!(s.len(), 3);
        assert_eq!(s.capacity(), 10);
        
        s.fill_to_capacity('c');
        
        // Now the string should have 'b' at the start, and 'c' to fill up to the capacity
        assert_eq!(s.len(), 10);
        assert_eq!(s.capacity(), 10);
        assert_eq!(s, "bbbccccccc");

        // Test when the string already has the same length as the capacity
        let mut s = String::with_capacity(5);
        s.push('d');
        s.push('d');
        s.push('d');
        s.push('d');
        s.push('d');
        
        assert_eq!(s.len(), 5);
        assert_eq!(s.capacity(), 5);
        
        s.fill_to_capacity('e');
        
        // No changes should happen as the string's length equals the capacity
        assert_eq!(s.len(), 5);
        assert_eq!(s.capacity(), 5);
        assert_eq!(s, "ddddd");
    }
}