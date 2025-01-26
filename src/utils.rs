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
