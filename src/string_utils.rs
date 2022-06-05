use core::panic;
use std::ops::{Bound, RangeBounds};
use std::iter;
use std::mem;
use std::collections::HashMap;

extern crate unic_normal;
use unic_normal::StrNormalForm;

pub trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> &str;
    fn slice(&self, range: impl RangeBounds<usize>) -> &str;
    fn get_vec_chars(&self) -> Vec<char>;
}

impl StringUtils for str {
    fn substring(&self, start: usize, len: usize) -> &str {
        let mut char_pos = 0;
        let mut byte_start = 0;
        let mut it = self.chars();
        loop {
            if char_pos == start { break; }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_start += c.len_utf8();
            }
            else { break; }
        }
        char_pos = 0;
        let mut byte_end = byte_start;
        loop {
            if char_pos == len { break; }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_end += c.len_utf8();
            }
            else { break; }
        }
        &self[byte_start..byte_end]
    }

    fn slice(&self, range: impl RangeBounds<usize>) -> &str {
        let start = match range.start_bound() {
            Bound::Included(bound) | Bound::Excluded(bound) => *bound,
            Bound::Unbounded => 0,
        };
        let len = match range.end_bound() {
            Bound::Included(bound) => *bound + 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.len(),
        } - start;
        self.substring(start, len)
    }

    fn get_vec_chars(&self) -> Vec<char> {
        //return self.chars().collect();
        let tmp_str = self.nfc().collect::<String>();
        tmp_str.chars().collect()
    }
}

pub trait StringUtilsVecChars {
    fn to_string(&self) -> String;
    fn to_string_buf<'a>(&self, buf: & 'a mut String) -> & 'a String;
}

impl StringUtilsVecChars for Vec<char> {
    fn to_string(&self) -> String { 
        self.iter().collect()
    }

    fn to_string_buf<'a>(&self, buf: & 'a mut String) -> & 'a String {
        buf.clear();
        for c in self.iter() {
            buf.push(*c);
        }
        buf
    }
}

pub trait StringUtilsSlices {
    fn to_string(&self) -> String;
    fn to_string_buf<'a>(&self, buf: & 'a mut String) -> & 'a String;
    fn to_vec_chars(&self) -> Vec<char>;
}

impl StringUtilsSlices for [char] {
    fn to_string(&self) -> String {
        self.iter().collect()
    }

    fn to_string_buf<'a>(&self, buf: & 'a mut String) -> & 'a String {
        buf.clear();
        for c in self.iter() {
            buf.push(*c);
        }
        buf
    }

    fn to_vec_chars(&self) -> Vec<char> {
        let vec_chars = self.iter().copied().collect();
        vec_chars
    }
}


pub trait StringUtilsVecCharsV2 {
    // fn to_string(&self) -> String;
    // fn to_string_buf<'a>(&self, buf: & 'a mut String) -> & 'a String;
    
    fn join_vec(p_vec_vec_chars: &[&[char]]) -> Vec<char>;
    fn join_str(p_vec_str: &[&str]) -> Vec<char>;
 
    fn eq_vec(&self, other: &[char]) -> bool;
    fn eq_str(&self, p_str: &str) -> bool;

    fn push_vec(& mut self, p_vec_chars: &[char]);
    fn push_str(& mut self, p_str: &str);
    fn push_str_start(& mut self, p_str: &str);
    fn push_vec_start(& mut self, other_vec: &Vec<char>);
    fn insert_str(& mut self, p_str: &str, at_pos: usize) -> Result<(), String>;
    fn insert_vec(& mut self, other_vec: &Vec<char>, at_pos: usize) -> Result<(), String>;

    fn trim_start(& mut self);
    fn trim_end(& mut self);
    fn trim(& mut self);

    fn find_vec(& self, p_vec_chars: &Vec<char>, start_pos: usize, end_pos: Option<usize>) -> Option<usize>;
    fn find_str(& self, p_str: &str, start_pos: usize, end_pos: Option<usize>) -> Option<usize>;

    fn contains_vec(& self, p_vec_chars: &Vec<char>) -> bool;
    fn contains_str(& self, p_str: &str) -> bool;

    fn start_with_vec(& self, pattern_vec_chars: &[char]) -> bool;
    fn start_with_str(& self, pattern_str: &str) -> bool;
    fn ends_with_vec(& self, pattern_vec_chars: &[char]) -> bool;
    fn ends_with_str(& self, pattern_str: &str) -> bool;

    fn find_vec_all(& self, pattern_vec_chars: &Vec<char>) -> Vec<usize>;
    fn find_str_all(& self, pattern_str: &str) -> Vec<usize>;

    /// Returns a None or the index of the first replace.
    fn replace_vec(& mut self, match_pattern_vec: &Vec<char>, replace_pattern_vec: &Vec<char>, start_pos: usize, end_pos: Option<usize>) -> Option<usize>;
    /// Returns a None or the index of the first replace.
    fn replace_str(& mut self, match_pattern_str: &str, replace_pattern_str: &str, start_pos: usize, end_pos: Option<usize>) -> Option<usize>;
    
    /// Returns a None or the number of replaces.
    fn replace_vec_all(& mut self, match_pattern_vec: &Vec<char>, replace_pattern_vec: &Vec<char>) -> Option<usize>;
    /// Returns a None or the number of replaces.
    fn replace_str_all(& mut self, match_pattern_str: &str, replace_pattern_str: &str) -> Option<usize>;
    
    fn split_vec(& self, at_pattern: &Vec<char>) -> Vec<&[char]>;        
    fn split_str(& self, at_pattern_str: &str) -> Vec<&[char]>;

    fn map_str(& mut self, map: & HashMap<&str, &str>) -> HashMap<String, usize>;
}

impl StringUtilsVecCharsV2 for Vec<char> {
    
/*
    fn to_string(&self) -> String { 
        self.iter().collect()
    }

    fn to_string_buf<'a>(&self, buf: & 'a mut String) -> & 'a String {
        buf.clear();
        for c in self.iter() {
            buf.push(*c);
        }
        buf
    }
*/

    fn join_vec(p_vec_vec_chars: &[&[char]]) -> Vec<char> {
        // Calculate the total length of the strings.
        let mut capacity = 0_usize;
        for src_vec_chars_tmp in p_vec_vec_chars {
            capacity += src_vec_chars_tmp.len();
        }
        // Allocate the memory on the heap.
        let mut vec_chars: Vec<char> = Vec::with_capacity(capacity);
        for src_vec_chars_tmp in p_vec_vec_chars {
            vec_chars.push_vec(src_vec_chars_tmp);
        }
        vec_chars
    }

    fn join_str(p_vec_str: &[&str]) -> Vec<char> {
        // Calculate the total length of the strings.
        let mut capacity = 0_usize;
        for str_tmp in p_vec_str {
            for _ in str_tmp.chars() {
                capacity += 1;
            }
        }
        // Allocate the memory on the heap.
        let mut vec_chars: Vec<char> = Vec::with_capacity(capacity);
        for str_tmp in p_vec_str {
            vec_chars.push_str(str_tmp);
        }
        vec_chars
    } 

    #[inline]
    fn eq_vec(&self, other: &[char]) -> bool {
        self.len() == other.len() && iter::zip(self, other).all(|(a, b)| *a == *b)
    }

    #[inline]
    fn eq_str(&self, p_str: &str) -> bool {
        self.len() == p_str.chars().count() && iter::zip(self, p_str.chars()).all(|(a, b)| *a == b)
    }

    fn push_vec(& mut self, p_vec_chars: &[char]) {
        self.extend(p_vec_chars);
    }
  
    fn push_str(& mut self, p_str: &str) {
        let vec_chars = p_str.get_vec_chars();
        self.extend(vec_chars);
    }
    
    fn push_str_start(& mut self, p_str: &str) {              
        let mut vec_chars = p_str.get_vec_chars();
        vec_chars.extend(self.iter());
        let _ = mem::replace(self, vec_chars);
    }

    fn push_vec_start(& mut self, other_vec: &Vec<char>) {
        let mut vec_tmp = other_vec.clone();
        vec_tmp.extend(self.iter());
        let _ = mem::replace(self, vec_tmp);
    }

    fn insert_str(& mut self, p_str: &str, at_pos: usize) -> Result<(), String> {
        if at_pos >= self.len() {
            return Err("Error: In insert_str(), parameter at_pos is greater then sel.len() - 1".to_string());
        }
        let vec_t1: Vec<char> = p_str.get_vec_chars();
        let mut vec_tmp: Vec<char> = Vec::with_capacity(self.len() + vec_t1.len());
        vec_tmp.extend(self[..at_pos].iter());
        vec_tmp.extend(vec_t1.iter());
        vec_tmp.extend(self[at_pos..].iter());
        let _ = mem::replace(self, vec_tmp);
        Ok(())
    }

    fn insert_vec(& mut self, other_vec: &Vec<char>, at_pos: usize) -> Result<(), String> {
        
        if at_pos >= self.len() {
            return Err("Error: In insert_str(), parameter at_pos is greater then sel.len() - 1".to_string());
        }
        //let mut vec_t1: Vec<char> = p_str.chars().collect();
        let mut vec_tmp: Vec<char> = Vec::with_capacity(self.len() + other_vec.len());
        vec_tmp.extend(self[..at_pos].iter());
        vec_tmp.extend(other_vec.iter());
        vec_tmp.extend(self[at_pos..].iter());
        let _ = mem::replace(self, vec_tmp);
        Ok(())
    }

    fn trim_start(& mut self) {
        if self.is_empty() {
            return;
        }
        let mut start_index = 0;
        let mut i = 0;
        while i < self.len() && self[i].is_whitespace() {
            start_index = i;
            i += 1;
        }
        if i < self.len() - 1 {
            self.copy_within((start_index + 1).., 0);
            for _ in 0..i {
                self.pop();
            }
        } else {
            self.clear();
        }
    }

    fn trim_end(& mut self) {
        let mut i = self.len() as i32 - 1;
        while i >= 0 && self[i as usize].is_whitespace() {
            let _ = self.pop();
            i -= 1;
        }
    }

    fn trim(& mut self) {
        let mut vec_tmp: Vec<char> = Vec::with_capacity(self.len());
        let mut flag_starting_white_spaces = true;
        for c in self.iter() {
            if !c.is_whitespace() {
                flag_starting_white_spaces = false;
            }
            if !flag_starting_white_spaces {
                vec_tmp.push(*c);
            }
        }
        vec_tmp.trim_end();
        let _ = mem::replace(self, vec_tmp);
    }


    fn find_vec(& self, p_vec_chars: &Vec<char>, start_pos: usize, end_pos: Option<usize>) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        if start_pos >= self.len() {
            panic!("Error: In find_str() parameter start_pos must not be greater then Vec<char>.len() - 1 .");
        }
        let end_pos_val = if let Some(val) = end_pos {
                if val >= self.len() {
                    panic!("Error: In find_str() parameter end_pos must not be greater then Vec<char>.len() - 1 .");
                }
                if val < start_pos {
                    panic!("Error: In find_str() parameter end_pos cannot be lower then parameter start_pos.");
                } 
                val
            } else {
                self.len() - 1
            };
        if p_vec_chars.is_empty() {
            return None;
        }
        // let pattern_vec: Vec<char> = p_str.chars().collect(); 
        let pattern_vec = p_vec_chars; 
        if  pattern_vec.len() + start_pos > self.len() {
            return None;
        }

        // Find pattern inside string.
        let match_pos: usize;
        // let mut flag_match = false;
        for i in start_pos..=end_pos_val {
            let mut counter = pattern_vec.len();
            let mut offset = 0_usize;
            for c in pattern_vec.iter() {
                if self[i + offset] != *c {
                    break;
                }
                offset += 1;
                counter -= 1
            }
            if counter == 0 {
                // flag_match = true;
                match_pos = i;
                return Some(match_pos);
            }
        }
        None
    }

    fn find_str(& self, p_str: &str, start_pos: usize, end_pos: Option<usize>) -> Option<usize> {
        let pattern_vec_chars: Vec<char> = p_str.get_vec_chars(); 
        self.find_vec(&pattern_vec_chars, start_pos, end_pos)
    }

    fn contains_vec(& self, p_vec_chars: &Vec<char>) -> bool {
        if self.find_vec(p_vec_chars, 0, None).is_some() {
            return true;
        }
        false
    }

    fn contains_str(& self, p_str: &str) -> bool {
        let vec_chars = p_str.get_vec_chars();
        self.contains_vec(&vec_chars)
    }



    fn start_with_vec(& self, pattern_vec_chars: &[char]) -> bool {
        if pattern_vec_chars.len() > self.len() {
            return false;
        }
        self.starts_with(pattern_vec_chars)
    }


    fn start_with_str(& self, pattern_str: &str) -> bool {
        let pattern_vec_chars = pattern_str.get_vec_chars();
        self.start_with_vec(&pattern_vec_chars)
    }

    fn ends_with_vec(& self, pattern_vec_chars: &[char]) -> bool {
        if pattern_vec_chars.len() > self.len() {
            return false;
        }
        self.ends_with(pattern_vec_chars)
    }

    fn ends_with_str(& self, pattern_str: &str) -> bool {
        let pattern_vec_chars = pattern_str.get_vec_chars();
        self.ends_with_vec(&pattern_vec_chars)
    }

    /// Returns a Vec<usize> with all the finds.
    fn find_vec_all(&self, pattern_vec_chars: &Vec<char>) -> Vec<usize> {
        let mut flag_ended_find = false;
        let mut next_start_pos = 0_usize;
        let mut indexes_vec: Vec<usize> = Vec::new(); 
        // Find, from start to end, the indexes of the machs. Put's them on a Vec.
        while !flag_ended_find {
            if next_start_pos >= self.len() {
                // flag_ended_find = true;
                break;
            }
            let res = self.find_vec(pattern_vec_chars, next_start_pos, None);
            if let Some(index) = res {
                indexes_vec.push(index);
                next_start_pos = index + pattern_vec_chars.len();                
            } else {
                flag_ended_find = true;
            }
        }
        indexes_vec
    }

    fn find_str_all(&self, pattern_str: &str) -> Vec<usize> {
        let pattern_vec_chars: Vec<char> = pattern_str.get_vec_chars(); 
        self.find_vec_all(&pattern_vec_chars)
    }

    /// Returns a None or the index of the first replace.
    fn replace_vec(& mut self, match_pattern_vec: &Vec<char>, replace_pattern_vec: &Vec<char>, start_pos: usize, end_pos: Option<usize>) -> Option<usize> {
        let res = self.find_vec(match_pattern_vec, start_pos, end_pos);
        if let Some(index) = res {
            for _ in 0..(match_pattern_vec.len()) {
                self.remove(index);
            }
            let _ = self.insert_vec(replace_pattern_vec, index);
            return Some(index);
        }
        None
    }

    /// Returns a None or the index of the first replace.
    fn replace_str(& mut self, match_pattern_str: &str, replace_pattern_str: &str, start_pos: usize, end_pos: Option<usize>) -> Option<usize> {
        let match_pattern_vec = match_pattern_str.get_vec_chars();
        let replace_pattern_vec = replace_pattern_str.get_vec_chars();
        self.replace_vec(&match_pattern_vec, &replace_pattern_vec, start_pos, end_pos)
    }

    /// Returns a None or the number of replaces.
    fn replace_vec_all(& mut self, match_pattern_vec: &Vec<char>, replace_pattern_vec: &Vec<char>) -> Option<usize> {
        let mut flag_ended_find = false;
        let mut next_start_pos = 0_usize;
        let mut indexes_vec: Vec<usize> = Vec::new(); 
        // Find, from start to end, the indexes of the machs. Put's them on a Vec.
        while !flag_ended_find {
            if next_start_pos >= self.len() {
                // flag_ended_find = true;
                break;
            }
            let res = self.find_vec(match_pattern_vec, next_start_pos, None);
            if let Some(index) = res {
                indexes_vec.push(index);
                next_start_pos = index + match_pattern_vec.len();                
            } else {
                flag_ended_find = true;
            }
        }
        // Case where it didn't found any match, it will exit earlier.
        if indexes_vec.is_empty() {
            return None;
        }

        // We will copy to a new Vec<char> the data and do the replacement when coping.
        let num_matches = indexes_vec.len();
        let capacity = self.len() - num_matches * match_pattern_vec.len() + num_matches * replace_pattern_vec.len(); 
        let mut target_vec_chars: Vec<char> = Vec::with_capacity(capacity);
        let mut last_index = 0_usize;
        for (counter,index) in indexes_vec.iter().enumerate() {
            // Copy the first chars before the first match.
            if last_index < self.len() {
                target_vec_chars.push_vec(&self[last_index..*index]);
                target_vec_chars.push_vec(replace_pattern_vec);
                last_index = index + match_pattern_vec.len();
                if counter == indexes_vec.len() - 1 && last_index < self.len() {
                    target_vec_chars.push_vec(&self[last_index..]);
                }
            }
        }

        // Copia a zona de memoria do src para o target self.
        let _ = mem::replace(self, target_vec_chars);

        Some(num_matches)
    }

    /// Returns a None or the number of replaces.
    fn replace_str_all(& mut self, match_pattern_str: &str, replace_pattern_str: &str) -> Option<usize> {
        self.replace_vec_all(&match_pattern_str.get_vec_chars(),
                           &replace_pattern_str.get_vec_chars())
    }

    fn split_vec(& self, at_pattern_vec: &Vec<char>) -> Vec<&[char]> {
        let match_pattern_vec = at_pattern_vec;
        let mut flag_ended_find = false;
        let mut next_start_pos = 0_usize;
        let mut indexes_vec: Vec<usize> = Vec::new(); 
        // Find, from start to end, the indexes of the machs. Put's them on a Vec.
        while !flag_ended_find {
            if next_start_pos >= self.len() {
                // flag_ended_find = true;
                break;
            }
            let res = self.find_vec(match_pattern_vec, next_start_pos, None);
            if let Some(index) = res {
                indexes_vec.push(index);
                next_start_pos = index + match_pattern_vec.len();                
            } else {
                flag_ended_find = true;
            }
        }

        let mut res_vec: Vec<&[char]> = Vec::new();
        // Case where it didn't found any match, it will exit earlier.
        if indexes_vec.is_empty() {
            return res_vec;
        }

        // Join the intervals between splits that have chars, that are not the split chars.
        let mut last_index = 0_usize;
        for (counter, index) in indexes_vec.iter().enumerate() {
            // Copy the first chars before the first match.
            if last_index < self.len() {
                let slice_tmp = &self[last_index..*index];
                if !slice_tmp.is_empty() {
                    res_vec.push(slice_tmp);
                }
                last_index = index + match_pattern_vec.len();
                if counter == indexes_vec.len() - 1 && last_index < self.len() {
                    let slice_tmp = &self[last_index..];
                    if !slice_tmp.is_empty() {
                        res_vec.push(slice_tmp);
                    }   
                }
            }
        }

        res_vec
    }

    fn split_str(& self, at_pattern_str: &str) -> Vec<&[char]> {
        self.split_vec(&at_pattern_str.get_vec_chars())
    }

    fn map_str(& mut self, map: & HashMap<&str, &str>) -> HashMap<String, usize> {
        let mut res_hashmap: HashMap<String, usize> = HashMap::new();
        for (src_str, target_str) in map.iter() {
            let res = self.replace_str_all(src_str, target_str);
            match res {
                Some(num_replaces_for_seg_string) => res_hashmap.insert(src_str.to_string(), num_replaces_for_seg_string),
                None => res_hashmap.insert(src_str.to_string(), 0_usize),
            };
        }
        res_hashmap
    }

}


