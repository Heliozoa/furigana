//! `furigana::map` is more accurate than `map_naive` by using kanji reading information to grade the mappings it produces.

use std::collections::HashMap;

fn main() {
    let mut kanji_to_readings = HashMap::new();
    kanji_to_readings.insert("物".to_string(), vec!["もの".to_string()]);
    kanji_to_readings.insert("怪".to_string(), vec!["け".to_string()]);
    let mapping = furigana::map("物の怪", "もののけ", &kanji_to_readings)
        .into_iter()
        .max_by_key(|f| f.accuracy)
        .unwrap();
    println!("{mapping}");
}
