pub fn is_cjk(data: &char) -> bool {
    match data {
        '\u{4E00}'..='\u{9FFF}' => true,
        '\u{3400}'..='\u{4DBF}' => true,
        '\u{20000}'..='\u{2A6DF}' => true,
        '\u{2A700}'..='\u{2B73F}' => true,
        '\u{2B740}'..='\u{2B81F}' => true,
        '\u{2B820}'..='\u{2CEAF}' => true,
        '\u{F900}'..='\u{FAFF}' => true,
        '\u{2F800}'..='\u{2FA1F}' => true,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_return_true() {
        let char = "半".chars().next().unwrap();
        let result = is_cjk(&char);
        assert!(result == true)
    }

    #[test]
    fn should_return_false() {
        let char = "a".chars().next().unwrap();
        let result = is_cjk(&char);
        assert!(result == false)
    }
}
