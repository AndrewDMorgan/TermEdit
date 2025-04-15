#[allow(dead_code)]

pub mod string_pattern_matching {
    // kinda bad but kinda actually works; at least it's really fast
    pub fn byte_comparison (wordMain: &String, wordComp: &String) -> usize {
        //if wordMain.is_empty() || wordComp.is_empty() {  return usize::MAX;  }
        let mut totalError = 0;
        let wordBytes = wordComp.as_bytes();
        for (index, byte) in wordMain
            .bytes()
            .enumerate()
        {
            if index >= wordComp.len() {  break;  }
            totalError += (byte as isize - wordBytes[index] as isize)
                .unsigned_abs();
        }

        if wordComp.len() < wordMain.len() {
            totalError += (wordMain.len() - wordComp.len()) * 2;
        }

        totalError
    }

    // too slow for practical use in this project :(
    // it seems fairly accurate and nice; far better than
    // the messy byte comparison
    fn levenshtein_recursive (word1: &String, word2: &String, i: usize, j: usize) -> usize {
        if i == 0 {  return j;  }
        if j == 0 {  return i;  }

        // i, j are always greater than 0 (above) and less than the strings' sizes
        let not_eql = unsafe {
            word1.get_unchecked(i-1..i) != word2.get_unchecked(j-1..j)
        };
        std::cmp::min(std::cmp::min(
            levenshtein_recursive(word1, word2, i - 1, j) + 1,
            levenshtein_recursive(word1, word2, i, j - 1) + 1),
            levenshtein_recursive(word1, word2, i - 1, j - 1) + {if not_eql {1} else {0}}
        )
    }

    pub fn levenshtein_distance (query_word: &String, other_word: &String) -> usize {
        levenshtein_recursive(query_word, other_word, query_word.len(), other_word.len())
    }

    // a wrapper that removes the extra i, j parameters
    pub fn levenshtein_distance_itter (query_word: &String, words: &Vec <String>) -> usize {
        let mut min_distance = usize::MAX;
        for word in words {
            let distance = levenshtein_recursive(query_word, word, query_word.len(), word.len());
            if distance < min_distance {
                min_distance = distance;
            }
        } min_distance
    }
}

