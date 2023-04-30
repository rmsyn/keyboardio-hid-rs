#![no_std]

// Rearranges the keys list so that the free (= 0x00) slots are at the
// end of the keys list - some implementations stop for keys at the
// first occurence of an 0x00 in the keys list.
//
// So (0x00)(0x01)(0x00)(0x03)(0x02)(0x00) becomes
//    (0x02)(0x01)(0x03)(0x00)(0x00)(0x00)
//
// Does not care about the order of non-zero key slots.
pub fn sort_keycodes(keys: &mut [u8]) {
    let len = keys.len();

    let mut front_idx = 0;
    let mut back_idx = len - 1;

    // Move leading zero key slots to the end of the list
    while front_idx < back_idx {
        if keys[front_idx] == 0 {
            // Search for a non-zero slot, starting at the back of the list.
            // Iterate backwards until a non-zero slot is found, or the front index is reached.
            while keys[back_idx] == 0 && back_idx > front_idx {
                back_idx -= 1;
            }

            xor_swap(keys, front_idx, back_idx);
        }
        front_idx += 1;
    }
}

pub fn xor_swap(slice: &mut [u8], left_idx: usize, right_idx: usize) {
    let len = slice.len();
    if left_idx < len && right_idx < len && slice[left_idx] != slice[right_idx] {
        // XOR the right value with the left value to get a mixed value
        slice[left_idx] ^= slice[right_idx];
        // XOR the mixed value with the original right value
        // This removes the original right value from the mix, leaving the left value
        slice[right_idx] ^= slice[left_idx];
        // XOR the mixed value with previous left value, leaving the right value
        slice[left_idx] ^= slice[right_idx];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_keycodes() {
        // so (0x00)(0x01)(0x00)(0x03)(0x02)(0x00) becomes
        //    (0x02)(0x01)(0x03)(0x00)(0x00)(0x00)
        let mut unsorted = [0x00, 0x01, 0x00, 0x03, 0x02, 0x00];
        let expected = [0x02, 0x01, 0x03, 0x00, 0x00, 0x00];
    
        sort_keycodes(&mut unsorted);
    
        assert_eq!(unsorted, expected);
    
        let mut unsorted = [0x01, 0x00, 0x00, 0x03, 0x00, 0x02];
        let expected = [0x01, 0x02, 0x03, 0x00, 0x00, 0x00];

        sort_keycodes(&mut unsorted);
    
        assert_eq!(unsorted, expected);

        let mut unsorted = [0x00, 0x00, 0x00, 0x03, 0x01, 0x02];
        let expected = [0x02, 0x01, 0x03, 0x00, 0x00, 0x00];

        sort_keycodes(&mut unsorted);
    
        assert_eq!(unsorted, expected);
    }
}
