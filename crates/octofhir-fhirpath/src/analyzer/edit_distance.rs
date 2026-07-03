/// Calculate Levenshtein distance between two strings.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let b_len = b_chars.len();

    if a_chars.is_empty() {
        return b_len;
    }
    if b_chars.is_empty() {
        return a_chars.len();
    }

    let mut previous: Vec<usize> = (0..=b_len).collect();
    let mut current = vec![0; b_len + 1];

    for (i, a_char) in a_chars.iter().enumerate() {
        current[0] = i + 1;

        for (j, b_char) in b_chars.iter().enumerate() {
            let substitution_cost = usize::from(a_char != b_char);
            current[j + 1] = std::cmp::min(
                std::cmp::min(previous[j + 1] + 1, current[j] + 1),
                previous[j] + substitution_cost,
            );
        }

        std::mem::swap(&mut previous, &mut current);
    }

    previous[b_len]
}
