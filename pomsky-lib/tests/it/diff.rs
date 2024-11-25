pub fn simple_diff<'a>(left: &'a str, right: &'a str) -> (&'a str, &'a str, &'a str, &'a str) {
    if left == right {
        return (left, "", "", "");
    }
    let ((prefix_len, _), _) =
        left.char_indices().zip(right.chars()).find(|&((_, a), b)| a != b).unwrap();

    let ((left_last_idx, _), _) =
        left.char_indices().rev().zip(right.chars().rev()).find(|&((_, a), b)| a != b).unwrap();

    let suffix_len = left.len() - left_last_idx - 1;
    let suffix_len = suffix_len.min(left.len() - prefix_len).min(right.len() - prefix_len);

    let prefix = &left[..prefix_len];
    let suffix = &left[left.len() - suffix_len..];

    let left_diff = &left[prefix_len..left.len() - suffix_len];
    let right_diff = &right[prefix_len..right.len() - suffix_len];

    (prefix, left_diff, right_diff, suffix)
}
