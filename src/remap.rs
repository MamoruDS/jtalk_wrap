use std::collections::HashMap;

pub fn char_remap(text: &String, char_map: HashMap<char, char>) -> String {
    text.chars()
        .map(|c| match char_map.get(&c) {
            Some(kanji) => *kanji,
            None => c,
        })
        .collect::<String>()
}
