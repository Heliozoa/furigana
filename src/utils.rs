//! Various utility functions

pub fn is_numeric_west(c: char) -> bool {
    c.is_numeric()
}

pub fn is_numeric_fullwidth(c: char) -> bool {
    ('０'..='９').contains(&c)
}

pub fn is_alphabetic(c: char) -> bool {
    c.is_ascii_alphabetic() || is_fullwidth(c) || is_halfwidth(c)
}

pub fn is_fullwidth(c: char) -> bool {
    ('Ａ'..='Ｚ').contains(&c) || ('ａ'..='ｚ').contains(&c)
}

pub fn is_halfwidth(c: char) -> bool {
    (0xFF61..=0xFF9F).contains(&(c as u32))
}

pub fn is_hiragana(c: char) -> bool {
    (0x3040..=0x309F).contains(&(c as u32))
}

pub fn is_katakana(c: char) -> bool {
    (0x30A0..=0x30FF).contains(&(c as u32))
}

pub fn is_kana(c: char) -> bool {
    is_hiragana(c) || is_katakana(c)
}

pub fn is_kanji(c: char) -> bool {
    (0x4E00..=0x9FFF).contains(&(c as u32))
}

// get valid readings for the digit by looking at the digit itself
// as well as the remaining number of digits after this one in the number
pub fn digit_readings(digit: char, numbers_left: usize) -> &'static [&'static str] {
    match digit {
        '0' | '０' => &["ぜろ", "れい"],
        '1' | '１' => {
            if numbers_left == 0 {
                &["いち", "ひと"]
            } else if numbers_left % 8 == 0 {
                &["おく", "いちおく"]
            } else {
                match (numbers_left - 1) % 4 {
                    3 => &["まん", "いちまん"],
                    2 => &["せん", "いっせん"],
                    1 => &["ひゃく", "いっぴゃく"],
                    0 => &["じゅう"],
                    _ => unreachable!(),
                }
            }
        }
        '2' | '２' => {
            if numbers_left == 0 {
                &["に", "ふた"]
            } else if numbers_left % 8 == 0 {
                &["におく"]
            } else {
                match (numbers_left - 1) % 4 {
                    3 => &["にまん"],
                    2 => &["にせん"],
                    1 => &["にひゃく"],
                    0 => &["にじゅう"],
                    _ => unreachable!(),
                }
            }
        }
        '3' | '３' => {
            if numbers_left == 0 {
                &["さん", "むっ"]
            } else if numbers_left % 8 == 0 {
                &["さんおく"]
            } else {
                match (numbers_left - 1) % 4 {
                    3 => &["さんまん"],
                    2 => &["さんぜん"],
                    1 => &["さんびゃく"],
                    0 => &["さんじゅう"],
                    _ => unreachable!(),
                }
            }
        }
        '4' | '４' => {
            if numbers_left == 0 {
                &["し", "よん", "よっ", "よ"]
            } else if numbers_left % 8 == 0 {
                &["よんおく"]
            } else {
                match (numbers_left - 1) % 4 {
                    3 => &["しまん", "よんまん"],
                    2 => &["しせん", "よんせん"],
                    1 => &["しひゃく", "よんひゃく"],
                    0 => &["しじゅう", "よんじゅう"],
                    _ => unreachable!(),
                }
            }
        }
        '5' | '５' => {
            if numbers_left == 0 {
                &["ご", "いつ"]
            } else if numbers_left % 8 == 0 {
                &["ごおく"]
            } else {
                match (numbers_left - 1) % 4 {
                    3 => &["ごまん"],
                    2 => &["ごせん"],
                    1 => &["ごひゃく"],
                    0 => &["ごじゅう"],
                    _ => unreachable!(),
                }
            }
        }
        '6' | '６' => {
            if numbers_left == 0 {
                &["ろく", "むっ"]
            } else if numbers_left % 8 == 0 {
                &["ろくおく"]
            } else {
                match (numbers_left - 1) % 4 {
                    3 => &["ろくまん"],
                    2 => &["ろくせん"],
                    1 => &["ろっぴゃく"],
                    0 => &["ろくじゅう"],
                    _ => unreachable!(),
                }
            }
        }
        '7' | '７' => {
            if numbers_left == 0 {
                &["しち", "なな"]
            } else if numbers_left % 8 == 0 {
                &["ななおく"]
            } else {
                match (numbers_left - 1) % 4 {
                    3 => &["しちまん", "ななまん"],
                    2 => &["しちせん", "ななせん"],
                    1 => &["しちひゃく", "ななひゃく"],
                    0 => &["しちじゅう", "ななじゅう"],
                    _ => unreachable!(),
                }
            }
        }
        '8' | '８' => {
            if numbers_left == 0 {
                &["はち", "やっ", "はっ"]
            } else {
                match (numbers_left - 1) % 4 {
                    3 => &["はちまん"],
                    2 => &["はっせん"],
                    1 => &["はっぴゃく"],
                    0 => &["はちじゅう"],
                    _ => unreachable!(),
                }
            }
        }
        '9' | '９' => {
            if numbers_left == 0 {
                &["きゅう", "ここの"]
            } else {
                match (numbers_left - 1) % 4 {
                    3 => &["きゅうまん"],
                    2 => &["きゅうせん"],
                    1 => &["きゅうひゃく"],
                    0 => &["きゅうじゅう"],
                    _ => unreachable!(),
                }
            }
        }
        _ => &[],
    }
}
