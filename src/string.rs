#![allow(dead_code)]

pub trait StringExt {
    fn fill_to_capacity(&mut self, c: char);
    fn insert_str_at(&mut self, x: usize, str: &str);
    fn replace_char_at(&mut self, x: usize, new_char: char);
}

impl StringExt for String {
    /// Fills the string from `length` to `capacity` with given character.
    fn fill_to_capacity(&mut self, c: char) {
        for _ in self.len()..self.capacity() {
            self.push(c);
        }
    }

    /// Replaces the characters in the string with `string` starting from `x`.
    /// If `x` is beyond the current length, pad with spaces (`' '`) until `x`.
    /// If the length of `string` goes beyond the current `String`'s capacity,
    /// then we stop pushing those characters.
    fn insert_str_at(&mut self, x: usize, string: &str) {
        let capacity = self.capacity();
        let len = self.len();

        // If `x` is greater than the current length, pad with spaces until the length reaches `x`
        if x > len {
            self.push_str(&" ".repeat(x - len)); // Pad with spaces
        }

        // Determine how many characters we can insert without exceeding the capacity
        let insert_len = string.len();
        let max_insert_len = capacity.saturating_sub(x) + 1;

        // We can only insert as many characters as will fit between the current length and capacity
        let insertable_len = std::cmp::min(insert_len, max_insert_len);

        // Replace the characters from position `x` or append characters as needed
        for i in 0..insertable_len {
            if x + i < self.len() {
                // If `x + i` is within the current string length, replace it
                self.replace_range(x + i..x + i + 1, &string[i..i + 1]);
            } else {
                // If `x + i` is beyond the current length, push the character
                self.push(string.chars().nth(i).unwrap());
            }
        }
    }
    
    // Replaces the character at the given index `x` in the string with `new_char`.
    /// If the index is out of bounds, the string remains unchanged.
    fn replace_char_at(&mut self, x: usize, new_char: char) {
        if x < self.len() {
            // Get the byte index of the character at index `x`
            let byte_index = self.char_indices().nth(x).map(|(idx, _)| idx);
            
            if let Some(byte_idx) = byte_index {
                // Replace the character at that byte index
                self.replace_range(byte_idx..byte_idx + new_char.len_utf8(), &new_char.to_string());
            }
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

    #[test]
    fn test_insert_str_at_empty_string() {
        // Test case 1: Inserting into an empty string
        let mut s = String::with_capacity(10);
        s.insert_str_at(0, "hello");
        assert_eq!(s, "hello");
        assert_eq!(s.capacity(), 10); // Capacity should remain 10
    }

    #[test]
    fn test_insert_str_at_non_empty_string() {
        // Test case 2: Inserting into a string with some content
        let mut s = String::with_capacity(10);
        s.push('a');
        s.push('b');
        s.push('c');
        s.insert_str_at(1, "12345");
        assert_eq!(s, "a12345");
        assert_eq!(s.capacity(), 10); // Capacity should remain the same
    }

    #[test]
    fn test_insert_str_at_beyond_capacity() {
        // Test case 3: Inserting beyond the capacity
        let mut s = String::with_capacity(10);
        s.push('x');
        s.push('y');
        s.push('z');
        s.insert_str_at(2, "abcdefghijklmnop");
        // Only the part that fits in the capacity should be inserted
        assert_eq!(s, "xyabcdefgh");
        assert_eq!(s.capacity(), 10); // Capacity remains the same
    }

    #[test]
    fn test_insert_str_at_greater_than_current_length() {
        // Test case 4: Inserting when index is greater than current length
        let mut s = String::with_capacity(5);
        s.push('a');
        s.insert_str_at(5, "xyz");
        // Since the current length is 1, it will be padded with spaces up to index 5
        assert_eq!(s, "a");
        assert_eq!(s.capacity(), 5); // The capacity remains the same
    }

    #[test]
    fn test_insert_str_at_index_larger_than_capacity() {
        // Test case 5: Inserting at index larger than the current capacity
        let mut s = String::with_capacity(5);
        s.push('a');
        s.insert_str_at(10, "hello");
        // The string will be padded with spaces up to index 10
        assert_eq!(s, "a");
        assert_eq!(s.capacity(), 5); // The capacity remains the same
    }

    #[test]
    fn test_insert_str_at_with_sufficient_capacity() {
        // Test case 6: Inserting at index with enough capacity
        let mut s = String::with_capacity(20);
        s.push('x');
        s.push('y');
        s.push('z');
        s.insert_str_at(7, "hello");
        // The string will be padded with spaces until index 7, then insert "hello"
        assert_eq!(s, "xyz    hello");
        assert_eq!(s.capacity(), 20); // Capacity remains the same
    }

    #[test]
    fn test_insert_str_at_with_exact_capacity() {
        // Test case 7: Inserting with exact capacity usage
        let mut s = String::with_capacity(10);
        s.push('a');
        s.push('b');
        s.insert_str_at(2, "cd");
        // String will now be "abc" + "cd", using all capacity
        assert_eq!(s, "abcd");
        assert_eq!(s.capacity(), 10); // Capacity remains the same
    }

    #[test]
    fn test_insert_str_at_no_space_for_insert() {
        // Test case 8: No space left for the insert
        let mut s = String::with_capacity(5);
        s.push('x');
        s.push('y');
        s.push('z');
        s.insert_str_at(2, "abcdefgh"); // More characters than capacity
        assert_eq!(s, "xyabc");
        assert_eq!(s.capacity(), 5); // Capacity remains the same, and only part of the insert fits
    }
}