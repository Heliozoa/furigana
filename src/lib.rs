#![doc = include_str!("../README.md")]

mod furigana;
mod segmentation;
mod utils;

pub use self::furigana::{Furigana, FuriganaNode, FuriganaSegment};
use self::furigana::{FuriganaTree, KanjiAccuracy};
use segmentation::{CoarseSegmentation, FineSegmentation, Segment};
use std::{collections::HashMap, iter::Peekable};

/// Returns a list of all possible ways to map the reading to the text, matching the kana in the reading to the ones in the word.
/// Returns an empty list if the segments and readings are impossible to match.
pub fn map_naive<'a>(text: &'a str, reading: &'a str) -> Vec<Furigana<'a>> {
    // no need to do work for kana words
    if text.chars().all(utils::is_kana) {
        return vec![Furigana {
            accuracy: 1,
            furigana: vec![FuriganaSegment {
                segment: text,
                furigana: None,
            }],
        }];
    }
    // no need to do work for single character words
    if text.chars().count() == 1 {
        return vec![Furigana {
            accuracy: 1,
            furigana: vec![FuriganaSegment {
                segment: text,
                furigana: Some(reading),
            }],
        }];
    }

    // need kanji information to assign readings to each individual kanji, so use coarse segmentation here
    let segments = CoarseSegmentation::new(text);
    // recurse with map_inner
    let nodes = map_inner(segments.peekable(), reading, None, None, false).unwrap_or_default();
    let tree = FuriganaTree {
        text,
        reading,
        nodes,
    };
    Furigana::from_tree(&tree)
}

/// Returns a list of all possible ways to map the reading to the text, matching the kana in the reading to the ones in the text.
/// Uses the information in `kanji_to_readings` to approximate the accuracy of each mapping.
/// Returns an empty list if the segments and readings are impossible to match.
pub fn map<'a>(
    word: &'a str,
    reading: &'a str,
    kanji_to_readings: &HashMap<String, Vec<String>>,
) -> Vec<Furigana<'a>> {
    // no need to do work for kana words
    if word.chars().all(utils::is_kana) {
        return vec![Furigana {
            accuracy: 1,
            furigana: vec![FuriganaSegment {
                segment: word,
                furigana: None,
            }],
        }];
    }
    // no need to do work for single character words
    if word.chars().count() == 1 {
        return vec![Furigana {
            accuracy: 1,
            furigana: vec![FuriganaSegment {
                segment: word,
                furigana: Some(reading),
            }],
        }];
    }

    // trying to assign a reading to each individual kanji, so use fine segmentation
    let segments = FineSegmentation::new(word);
    // recurse with map_inner
    let nodes = map_inner(
        segments.peekable(),
        reading,
        Some(kanji_to_readings),
        None,
        false,
    )
    .unwrap_or_default();
    let tree = FuriganaTree {
        text: word,
        reading,
        nodes,
    };
    Furigana::from_tree(&tree)
}

// recursive function that explores different branches of ways to assign the reading to the text
// short-circuits by returning None on invalid mappings
fn map_inner<'a, I>(
    // the remaining segments
    mut segments_rest: Peekable<I>,
    // the remaining reading
    reading_rest: &'a str,
    // mapping from kanji to its valid readings
    kanji_to_readings: Option<&HashMap<String, Vec<String>>>,
    // the previous kanji character, if any
    previous_kanji: Option<&'a str>,
    // indicates that if the next segment is a kanji,
    // its reading may be affected by rendaku
    // this is true when the previous segment is kana or kanji
    can_be_rendaku: bool,
) -> Option<Vec<FuriganaNode<'a>>>
where
    I: Iterator<Item = Segment<'a>> + Clone,
{
    match segments_rest.next() {
        Some(segment @ Segment::Kana(kana)) => {
            // try to get matching kana from reading
            let reading = reading_rest.get(0..kana.len())?;
            if !kana_equivalent(reading, kana) {
                // invalid mapping: segment and reading don't match
                return None;
            }
            let reading_rest = &reading_rest[kana.len()..];
            let extensions = map_inner(segments_rest, reading_rest, kanji_to_readings, None, true)?;
            Some(vec![FuriganaNode {
                segment,
                reading,
                extensions,
                kanji_accurate: None,
            }])
        }
        Some(segment @ Segment::Kanji(kanji)) => {
            // special casing...
            if kanji == "大" && reading_rest.starts_with("おとな") {
                if let Some(Segment::Kanji("人")) = segments_rest.peek() {
                    segments_rest.next();
                    let reading_rest = &reading_rest["おとな".len()..];
                    let extensions =
                        map_inner(segments_rest, reading_rest, kanji_to_readings, None, true)?;
                    return Some(vec![FuriganaNode {
                        segment: Segment::Kanji("大人"),
                        reading: "おとな",
                        extensions,
                        kanji_accurate: Some(KanjiAccuracy::Accurate),
                    }]);
                } else {
                    return None;
                }
            }

            // get valid readings for the kanji, if any were provided
            let kanji_readings = kanji_to_readings
                .and_then(|km| km.get(kanji))
                .map(Vec::as_slice);
            // try matching different lengths of the reading to the word
            let mut nodes = vec![];
            let reading_chars = reading_rest.chars().count();
            // no characters left in reading, invalid mapping
            if reading_chars == 0 {
                return None;
            }

            // try to use up varying lengths of the remaining reading
            let mut use_reading_up_to = 0;
            for reading_char in reading_rest.chars() {
                use_reading_up_to += reading_char.len_utf8();
                let reading = &reading_rest[..use_reading_up_to];

                let mut segments_rest = segments_rest.clone();
                let can_be_sokuonbin = segments_rest.peek().is_some();
                let reading_rest = &reading_rest[use_reading_up_to..];

                // keep recursing with the remaining segments and reading
                if let Some(extensions) = map_inner(
                    segments_rest,
                    reading_rest,
                    kanji_to_readings,
                    Some(kanji),
                    true,
                ) {
                    let kanji_accurate = check_kanji_accuracy(
                        kanji_readings,
                        reading,
                        can_be_rendaku,
                        can_be_sokuonbin,
                    );
                    nodes.push(FuriganaNode {
                        segment,
                        reading,
                        extensions,
                        kanji_accurate,
                    })
                }
            }
            if nodes.is_empty() {
                // found no valid continuations
                None
            } else {
                Some(nodes)
            }
        }
        Some(segment @ Segment::Alphabetic(alpha)) => {
            let alpha_reading = match alpha {
                "A" | "a" | "Ａ" | "ａ" => "エー",
                "B" | "b" | "Ｂ" | "ｂ" => "ビー",
                "C" | "c" | "Ｃ" | "ｃ" => "シー",
                "D" | "d" | "Ｄ" | "ｄ" => "ディー",
                "E" | "e" | "Ｅ" | "ｅ" => "イー",
                "F" | "f" | "Ｆ" | "ｆ" => "エフ",
                "G" | "g" | "Ｇ" | "ｇ" => "ギー",
                "H" | "h" | "Ｈ" | "ｈ" => "エイチ",
                "I" | "i" | "Ｉ" | "ｉ" => "アイ",
                "J" | "j" | "Ｊ" | "ｊ" => "ジェー",
                "K" | "k" | "Ｋ" | "ｋ" => "ケー",
                "L" | "l" | "Ｌ" | "ｌ" => "エル",
                "M" | "m" | "Ｍ" | "ｍ" => "エム",
                "N" | "n" | "Ｎ" | "ｎ" => "エヌ",
                "O" | "o" | "Ｏ" | "ｏ" => "オー",
                "P" | "p" | "Ｐ" | "ｐ" => "ピー",
                "Q" | "q" | "Ｑ" | "ｑ" => "キュー",
                "R" | "r" | "Ｒ" | "ｒ" => "アール",
                "S" | "s" | "Ｓ" | "ｓ" => "エス",
                "T" | "t" | "Ｔ" | "ｔ" => "ティー",
                "U" | "u" | "Ｕ" | "ｕ" => "ユー",
                "V" | "v" | "Ｖ" | "ｖ" => "ブイ",
                "W" | "w" | "Ｗ" | "ｗ" => "ダブルユー",
                "X" | "x" | "Ｘ" | "ｘ" => "エックス",
                "Y" | "y" | "Ｙ" | "ｙ" => "ワイ",
                "Z" | "z" | "Ｚ" | "ｚ" => "ゼット",
                _ => unreachable!("unexpected alphabetic character '{alpha}'"),
            };

            // check if the letter's reading and the next section of the reading and are kana equvalent
            let corresponding_reading = reading_rest.get(..alpha_reading.len())?;
            if kana_equivalent(corresponding_reading, alpha_reading) {
                let reading_rest = &reading_rest[alpha_reading.len()..];
                let extensions =
                    map_inner(segments_rest, reading_rest, kanji_to_readings, None, true)?;
                return Some(vec![FuriganaNode {
                    segment,
                    reading: alpha_reading,
                    extensions,
                    kanji_accurate: None,
                }]);
            }
            None
        }
        Some(segment @ Segment::Numeric(number)) => {
            if number == "10" || number == "１０" && reading_rest.starts_with("とお") {
                // special casing とお for １０
                let reading_rest = &reading_rest["とお".len()..];
                let extensions =
                    map_inner(segments_rest, reading_rest, kanji_to_readings, None, false)?;
                return Some(vec![FuriganaNode {
                    segment,
                    reading: "とお",
                    extensions,
                    kanji_accurate: Some(KanjiAccuracy::Accurate),
                }]);
            }

            let mut readings = Vec::new();
            let mut numbers_left = number.chars().count();
            for digit in number.chars() {
                numbers_left -= 1;

                let digit_readings = utils::digit_readings(digit, numbers_left);
                if readings.is_empty() {
                    // initialise the readings list with the potential readings for the first digit
                    for &digit_reading in digit_readings {
                        if reading_rest.starts_with(digit_reading) {
                            readings.push(digit_reading);
                        }
                    }
                } else {
                    // else, we'll tack on each potential reading for this digit onto each potential reading
                    // for the preceding digit sequence
                    let mut new_readings = Vec::new();
                    for &reading in &readings {
                        // cut off the reading already processed
                        let reading_remaining = &reading_rest[reading.len()..];
                        let mut valid_digit_readings = Vec::new();
                        for &digit_reading in digit_readings {
                            if reading_remaining.starts_with(digit_reading) {
                                valid_digit_readings.push(digit_reading);
                            }
                        }

                        if valid_digit_readings.is_empty() {
                            // having no reading for zero is okay
                            if digit == '0' || digit == '０' {
                                new_readings.push(reading);
                            } else {
                                // else, this reading branch is invalid
                                continue;
                            }
                        } else {
                            for valid_digit_reading in &valid_digit_readings {
                                new_readings.push(
                                    &reading_rest[..reading.len() + valid_digit_reading.len()],
                                );
                            }
                        }
                    }
                    readings = new_readings;
                }
            }

            // for all the readings we found, recurse
            let mut nodes = Vec::new();
            for reading in readings {
                let extensions = map_inner(
                    segments_rest.clone(),
                    &reading_rest[reading.len()..],
                    kanji_to_readings,
                    None,
                    false,
                )?;
                nodes.push(FuriganaNode {
                    segment,
                    reading: &reading_rest[..reading.len()],
                    extensions,
                    kanji_accurate: None,
                })
            }
            if nodes.is_empty() {
                None
            } else {
                Some(nodes)
            }
        }
        Some(segment @ Segment::Exception(exception)) => match exception {
            // ヶ is a special case that is always read as か
            "ヶ" => {
                let corresponding_reading_len = 'か'.len_utf8();
                let reading = reading_rest.get(..corresponding_reading_len)?;
                if reading == "か" {
                    let extensions = map_inner(
                        segments_rest,
                        &reading_rest[corresponding_reading_len..],
                        kanji_to_readings,
                        previous_kanji,
                        can_be_rendaku,
                    )?;
                    Some(vec![FuriganaNode {
                        segment,
                        reading,
                        extensions,
                        kanji_accurate: None,
                    }])
                } else {
                    None
                }
            }
            _ => None,
        },
        Some(segment @ Segment::Other(other)) => {
            match other {
                "々" => {
                    if let Some(kanji) = previous_kanji {
                        let kanji_readings = kanji_to_readings
                            .and_then(|km| km.get(kanji))
                            .map(Vec::as_slice);
                        // try matching different lengths of the reading to the word
                        let mut nodes = vec![];
                        let chars = reading_rest.chars().count();
                        if chars == 0 {
                            return None;
                        }
                        for chars in 1..=chars {
                            let chars_len =
                                reading_rest.chars().take(chars).map(char::len_utf8).sum();
                            let reading = &reading_rest[..chars_len];

                            let mut segments_rest = segments_rest.clone();
                            let can_be_sokuonbin = segments_rest.peek().is_some();
                            let reading_rest = &reading_rest[chars_len..];
                            if let Some(extensions) = map_inner(
                                segments_rest,
                                reading_rest,
                                kanji_to_readings,
                                Some(kanji),
                                true,
                            ) {
                                let kanji_accurate = check_kanji_accuracy(
                                    kanji_readings,
                                    reading,
                                    can_be_rendaku,
                                    can_be_sokuonbin,
                                );

                                nodes.push(FuriganaNode {
                                    segment,
                                    reading,
                                    extensions,
                                    kanji_accurate,
                                })
                            }
                        }
                        Some(nodes)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        None => {
            // out of segments
            if reading_rest.is_empty() {
                // valid: out of input
                Some(vec![])
            } else {
                // invalid: remaining readings can't be mapped to anything
                None
            }
        }
    }
}

// checks whether the strings are equivalent if ignoring the difference between hiragana and katakana
fn kana_equivalent(left: &str, right: &str) -> bool {
    const UNICODE_KANA_TABLE_DISTANCE: u32 = 96;
    let mut previous_left = None;
    let mut previous_right = None;
    for (left, right) in left.chars().zip(right.chars()) {
        if left == 'ー' && right != 'ー' {
            let Some(previous_right) = previous_right else {
                return false;
            };
            if !is_extension(previous_right, right) {
                return false;
            }
        } else if right == 'ー' && left != 'ー' {
            let Some(previous_left) = previous_left else {
                return false;
            };
            if !is_extension(previous_left, left) {
                return false;
            }
        }

        let leftu32 = left as u32;
        let rightu32 = right as u32;
        let equivalent = leftu32 == rightu32
            || leftu32 == rightu32 + UNICODE_KANA_TABLE_DISTANCE
            || leftu32 == rightu32 - UNICODE_KANA_TABLE_DISTANCE;
        if !equivalent {
            return false;
        }
        previous_left = Some(left);
        previous_right = Some(right);
    }
    left.chars().count() == right.chars().count()
}

// checks if the next char can be an "extension" of the previous char the same way ー is used for katakana.
fn is_extension(previous: char, next: char) -> bool {
    // this is confusing to look at, but just find the comma for each line and it'll make sense...
    matches!(
        (previous, next),
        ('あ' | 'か' | 'さ' | 'た' | 'な' | 'は' | 'ま', 'あ' | 'ア')
            | ('い' | 'き' | 'し' | 'ち' | 'に' | 'ひ' | 'み', 'い' | 'イ')
            | ('う' | 'く' | 'す' | 'つ' | 'ぬ' | 'ふ' | 'む', 'う' | 'ウ')
            | ('え' | 'け' | 'せ' | 'て' | 'ね' | 'へ' | 'め', 'え' | 'エ')
            | ('お' | 'こ' | 'そ' | 'と' | 'の' | 'ほ' | 'も', 'お' | 'オ')
    )
}

// checks if the actual reading could be the "ideal" reading (according to kanji reading info) with rendaku
fn rendaku_equivalent(ideal_reading: &str, actual_reading: &str) -> bool {
    let Some(ideal_char) = ideal_reading.chars().next() else {
        // both empty
        return actual_reading.is_empty();
    };
    let Some(actual_char) = actual_reading.chars().next() else {
        // ideal not empty, actual empty
        return false;
    };

    // rendaku only applies to the first character of the reading (probably?)
    let first_chars_rendaku_accurate = ideal_char == actual_char
        || match (ideal_char, actual_char) {
            // ka
            ('か' | 'カ', 'が' | 'ガ') => true,
            ('き' | 'キ', 'ぎ' | 'ギ') => true,
            ('く' | 'ク', 'ぐ' | 'グ') => true,
            ('け' | 'ケ', 'げ' | 'ゲ') => true,
            ('こ' | 'コ', 'ご' | 'ゴ') => true,
            // sa
            ('さ' | 'サ', 'ざ' | 'ザ') => true,
            ('し' | 'シ', 'じ' | 'ジ') => true,
            ('す' | 'ス', 'ず' | 'ズ') => true,
            ('せ' | 'セ', 'ぜ' | 'ゼ') => true,
            ('そ' | 'ソ', 'ぞ' | 'ゾ') => true,
            // ta
            ('た' | 'タ', 'だ' | 'ダ') => true,
            ('ち' | 'チ', 'ぢ' | 'ヂ') => true,
            ('つ' | 'ツ', 'づ' | 'ヅ') => true,
            ('て' | 'テ', 'で' | 'デ') => true,
            ('と' | 'ト', 'ど' | 'ド') => true,
            // ha
            ('は' | 'ハ', 'ば' | 'ぱ' | 'バ' | 'パ') => true,
            ('ひ' | 'ヒ', 'び' | 'ぴ' | 'ビ' | 'ピ') => true,
            ('ふ' | 'フ', 'ぶ' | 'ぷ' | 'ブ' | 'プ') => true,
            ('へ' | 'ヘ', 'べ' | 'ぺ' | 'ベ' | 'ペ') => true,
            ('ほ' | 'ホ', 'ぼ' | 'ぽ' | 'ボ' | 'ポ') => true,
            _ => false,
        };
    first_chars_rendaku_accurate
        && kana_equivalent(
            &ideal_reading[ideal_char.len_utf8()..],
            &actual_reading[actual_char.len_utf8()..],
        )
}

// checks if the actual reading could be the "ideal" reading (according to kanji reading info) with
// "sokuonbin" (consonant doubling)
fn sokuonbin_equivalent(ideal_reading: &str, actual_reading: &str) -> bool {
    let Some(ideal_char) = ideal_reading.chars().last() else {
        // both empty
        return actual_reading.is_empty();
    };
    let Some(actual_char) = actual_reading.chars().last() else {
        // ideal not empty, actual empty
        return false;
    };

    // sokuonbin only applies to the end of a reading
    let last_chars_sokuonbin_accurate = if ideal_char == actual_char {
        true
    } else {
        matches!((ideal_char, actual_char), ('く' | 'ち' | 'つ', 'っ'))
    };
    last_chars_sokuonbin_accurate
        && kana_equivalent(
            &ideal_reading[..ideal_reading.len() - ideal_char.len_utf8()],
            &actual_reading[..actual_reading.len() - actual_char.len_utf8()],
        )
}

// checks if the kanji reading is accurate according to the possible kanji readings
fn check_kanji_accuracy(
    kanji_readings: Option<&[String]>,
    kanji_reading: &str,
    can_be_rendaku: bool,
    can_be_sokuonbin: bool,
) -> Option<KanjiAccuracy> {
    let kanji_readings = kanji_readings?;
    let kanji_accurate = kanji_readings
        .iter()
        .any(|kr| kana_equivalent(kr, kanji_reading));
    if kanji_accurate {
        return Some(KanjiAccuracy::Accurate);
    }

    let rendaku_accurate = can_be_rendaku
        && kanji_readings
            .iter()
            .any(|kr| rendaku_equivalent(kr, kanji_reading));
    if rendaku_accurate {
        return Some(KanjiAccuracy::AccurateWithRendaku);
    }

    let sokuonbin_accurate = can_be_sokuonbin
        && kanji_readings
            .iter()
            .any(|kr| sokuonbin_equivalent(kr, kanji_reading));
    if sokuonbin_accurate {
        return Some(KanjiAccuracy::AccurateWithSokuonbin);
    }

    Some(KanjiAccuracy::Inaccurate)
}

#[cfg(test)]
mod test {
    use super::*;

    fn prepare_furigana(furigana: Vec<Furigana<'_>>) -> Vec<(i32, Vec<(&str, Option<&str>)>)> {
        furigana
            .into_iter()
            .map(|f| {
                (
                    f.accuracy,
                    f.furigana
                        .into_iter()
                        .map(|f| (f.segment, f.furigana))
                        .collect::<Vec<_>>(),
                )
            })
            .collect()
    }

    #[test]
    fn segments_naive() {
        let furigana = prepare_furigana(crate::map_naive("物の怪", "もののけ"));
        println!("{furigana:?}");
        assert!(furigana.contains(&(
            0,
            vec![("物", Some("もの")), ("の", None), ("怪", Some("け"))]
        )));
        assert!(furigana.contains(&(
            0,
            vec![("物", Some("も")), ("の", None), ("怪", Some("のけ"))],
        )));
        assert_eq!(furigana.len(), 2);
    }

    #[test]
    fn segments_with_kanji() {
        let mut kanji_to_readings = HashMap::new();
        kanji_to_readings.insert("物".to_string(), vec!["もの".to_string()]);
        kanji_to_readings.insert("怪".to_string(), vec!["け".to_string()]);
        let furigana = prepare_furigana(crate::map("物の怪", "もののけ", &kanji_to_readings));
        println!("{furigana:?}");

        assert!(furigana.contains(&(
            4,
            vec![("物", Some("もの")), ("の", None), ("怪", Some("け"))],
        )));
        assert!(furigana.contains(&(
            -4,
            vec![("物", Some("も")), ("の", None), ("怪", Some("のけ"))],
        )));
        assert_eq!(furigana.len(), 2);
    }

    #[test]
    fn single_kanji_word() {
        let mut kanji_to_readings = HashMap::new();
        kanji_to_readings.insert("一".to_string(), vec!["いち".to_string()]);
        let furigana = prepare_furigana(crate::map("一", "いち", &kanji_to_readings));
        println!("{furigana:?}");

        assert!(furigana.contains(&(1, vec![("一", Some("いち"))],)));
        assert_eq!(furigana.len(), 1);
    }

    #[test]
    fn handles_alphabet() {
        let furigana = prepare_furigana(crate::map(
            "CDプレイヤー",
            "シーディープレイヤー",
            &HashMap::new(),
        ));
        println!("{furigana:?}");

        assert!(furigana.contains(&(
            0,
            vec![
                ("C", Some("シー")),
                ("D", Some("ディー")),
                ("プレイヤー", None)
            ]
        )));
        assert_eq!(furigana.len(), 1);

        let furigana = prepare_furigana(crate::map_naive("CDプレイヤー", "シーディープレイヤー"));
        println!("{furigana:?}");

        assert!(furigana.contains(&(
            0,
            vec![
                ("C", Some("シー")),
                ("D", Some("ディー")),
                ("プレイヤー", None)
            ]
        )));
        assert_eq!(furigana.len(), 1);
    }

    #[test]
    fn handles_々() {
        let furigana = prepare_furigana(crate::map_naive("日々", "ひび"));
        println!("{furigana:?}");

        assert!(furigana.contains(&(0, vec![("日々", Some("ひび"))])));
        assert_eq!(furigana.len(), 1);

        let mut kanji_to_readings = HashMap::new();
        kanji_to_readings.insert("日".to_string(), vec!["ひ".to_string(), "び".to_string()]);
        let furigana = prepare_furigana(crate::map("日々", "ひび", &kanji_to_readings));
        println!("{furigana:?}");

        assert!(furigana.contains(&(4, vec![("日", Some("ひ")), ("々", Some("び"))])));
        assert_eq!(furigana.len(), 1);
    }

    #[test]
    fn kana_insensitive() {
        let furigana = prepare_furigana(crate::map_naive("離れる", "ハナレル"));
        println!("{furigana:?}");

        assert!(furigana.contains(&(0, vec![("離", Some("ハナ")), ("れる", None)])));
        assert_eq!(furigana.len(), 1);
    }

    #[test]
    fn handles_rendaku() {
        let mut kanji_to_readings = HashMap::new();
        kanji_to_readings.insert("花".to_string(), vec!["はな".to_string()]);
        kanji_to_readings.insert("火".to_string(), vec!["ひ".to_string()]);
        let furigana = prepare_furigana(crate::map("花火", "はなび", &kanji_to_readings));
        println!("{furigana:?}");

        assert!(furigana.contains(&(3, vec![("花", Some("はな")), ("火", Some("び"))])));
        assert_eq!(furigana.len(), 2);
    }

    #[test]
    fn handles_sokuonbin() {
        let mut kanji_to_readings = HashMap::new();
        kanji_to_readings.insert("格".to_string(), vec!["かく".to_string()]);
        kanji_to_readings.insert("好".to_string(), vec!["こう".to_string()]);
        let furigana = prepare_furigana(crate::map("格好", "かっこう", &kanji_to_readings));
        println!("{furigana:?}");

        assert!(furigana.contains(&(3, vec![("格", Some("かっ")), ("好", Some("こう"))])));
        assert_eq!(furigana.len(), 3);
    }

    #[test]
    fn handles_rendaku_and_sokuonbin() {
        let mut kanji_to_readings = HashMap::new();
        kanji_to_readings.insert("突".to_string(), vec!["とつ".to_string()]);
        kanji_to_readings.insert("破".to_string(), vec!["は".to_string()]);
        let furigana = prepare_furigana(crate::map("突破", "とっぱ", &kanji_to_readings));
        println!("{furigana:?}");

        assert!(furigana.contains(&(2, vec![("突", Some("とっ")), ("破", Some("ぱ"))])));
        assert_eq!(furigana.len(), 2);
    }

    #[test]
    fn handles_exceptions() {
        let furigana = prepare_furigana(crate::map_naive("一ヶ月", "いっかげつ"));
        println!("{furigana:?}");

        assert!(furigana.contains(&(
            0,
            vec![
                ("一", Some("いっ")),
                ("ヶ", Some("か")),
                ("月", Some("げつ"))
            ]
        )));
        assert_eq!(furigana.len(), 1);
    }

    #[test]
    //#[ignore = "todo"]
    fn handles_numbers() {
        let furigana = prepare_furigana(crate::map_naive("1010日", "せんじゅうにち"));
        println!("{furigana:?}");
        assert_eq!(furigana.len(), 1);
        assert_eq!(
            furigana[0].1,
            vec![("1010", Some("せんじゅう")), ("日", Some("にち"))]
        );

        let furigana = prepare_furigana(crate::map_naive(
            "12345日",
            "いちまんにせんさんびゃくよんじゅうごにち",
        ));
        println!("{furigana:?}");
        assert_eq!(furigana.len(), 1);
        assert_eq!(
            furigana[0].1,
            vec![
                ("12345", Some("いちまんにせんさんびゃくよんじゅうご")),
                ("日", Some("にち"))
            ]
        );

        let furigana = prepare_furigana(crate::map_naive(
            "123456789日",
            "いちおくにせんさんびゃくよんじゅうごまんろくせんななひゃくはちじゅうきゅうにち",
        ));
        println!("{furigana:?}");
        assert_eq!(furigana.len(), 1);
        assert_eq!(
            furigana[0].1,
            vec![
                ("123456789", Some("いちおくにせんさんびゃくよんじゅうごまんろくせんななひゃくはちじゅうきゅう")),
                ("日", Some("にち"))
            ]
        );
    }

    #[test]
    fn handles_irregular_otona() {
        let mut kanji_to_readings = HashMap::new();
        kanji_to_readings.insert(
            "大".to_string(),
            vec!["おお".to_string(), "ダイ".to_string(), "タイ".to_string()],
        );
        kanji_to_readings.insert(
            "人".to_string(),
            vec![
                "ひと".to_string(),
                "り".to_string(),
                "と".to_string(),
                "ジン".to_string(),
                "ニン".to_string(),
            ],
        );
        let furigana = prepare_furigana(crate::map("大人", "おとな", &kanji_to_readings));
        println!("{furigana:?}");

        assert!(furigana.contains(&(2, vec![("大人", Some("おとな"))])));
        assert_eq!(furigana.len(), 1);
    }

    #[test]
    fn handles_irregular_tooka() {
        let mut kanji_to_readings = HashMap::new();
        kanji_to_readings.insert("日".to_string(), vec!["にち".to_string(), "か".to_string()]);
        let furigana = prepare_furigana(crate::map("１０日", "とおか", &kanji_to_readings));
        println!("{furigana:?}");

        assert!(furigana.contains(&(4, vec![("１０", Some("とお")), ("日", Some("か"))])));
        assert_eq!(furigana.len(), 1);
    }
}
