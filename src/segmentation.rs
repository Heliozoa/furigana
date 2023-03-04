//! Contains iterators that segment a Japanese word.

use crate::utils;

/// Segment of a Japanese word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Segment<'a> {
    Kana(&'a str),
    Kanji(&'a str),
    Alphanumeric(&'a str),
    Exception(&'a str),
    Other(&'a str),
}

impl<'a> Segment<'a> {
    /// Returns the inner string.
    pub fn inner(self) -> &'a str {
        match self {
            Self::Kana(kana) => kana,
            Self::Kanji(kanji) => kanji,
            Self::Alphanumeric(alpha) => alpha,
            Self::Exception(exception) => exception,
            Self::Other(other) => other,
        }
    }
}

/// Iterator over a word's sequences of kanji and kana.
/// Differs from `FineSegmentation` in that sequences of kanji are considered single segments.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoarseSegmentation<'a> {
    rest: &'a str,
}

impl<'a> CoarseSegmentation<'a> {
    pub fn new(word: &'a str) -> Self {
        Self { rest: word }
    }
}

impl<'a> Iterator for CoarseSegmentation<'a> {
    type Item = Segment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.rest.chars().next()?;
        match classify_char(next) {
            Char::Alphanumeric => {
                let next_len = next.len_utf8();
                let next = &self.rest[..next_len];
                self.rest = &self.rest[next_len..];
                Some(Segment::Alphanumeric(next))
            }
            Char::Exception => {
                let next_len = next.len_utf8();
                let next = &self.rest[..next_len];
                self.rest = &self.rest[next_len..];
                Some(Segment::Exception(next))
            }
            Char::Other => {
                let next_len = next.len_utf8();
                let next = &self.rest[..next_len];
                self.rest = &self.rest[next_len..];
                Some(Segment::Other(next))
            }
            Char::Kanji => {
                let next = if let Some(idx) = self
                    .rest
                    // 々 is special cased so that it can be included in a kanji segment
                    .find(|c| classify_char(c) != Char::Kanji && c != '々')
                {
                    let next = &self.rest[..idx];
                    self.rest = &self.rest[idx..];
                    next
                } else {
                    let next = self.rest;
                    self.rest = "";
                    next
                };
                Some(Segment::Kanji(next))
            }
            Char::Kana => {
                let next = if let Some(idx) = self.rest.find(|c| classify_char(c) != Char::Kana) {
                    let next = &self.rest[..idx];
                    self.rest = &self.rest[idx..];
                    next
                } else {
                    let next = self.rest;
                    self.rest = "";
                    next
                };
                Some(Segment::Kana(next))
            }
        }
    }
}

/// Iterator over a word's kanji and sequences of kana.
/// Differs from `CoarseSegmentation` in that each kanji is its own segment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FineSegmentation<'a> {
    rest: &'a str,
}

impl<'a> FineSegmentation<'a> {
    pub fn new(word: &'a str) -> Self {
        Self { rest: word }
    }
}

impl<'a> Iterator for FineSegmentation<'a> {
    type Item = Segment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.rest.chars().next()?;
        let next_class = classify_char(next);
        match next_class {
            Char::Alphanumeric => {
                let next_len = next.len_utf8();
                let next = &self.rest[..next_len];
                self.rest = &self.rest[next_len..];
                Some(Segment::Alphanumeric(next))
            }
            Char::Kanji => {
                let idx = next.len_utf8();
                let next = &self.rest[..idx];
                self.rest = &self.rest[idx..];
                Some(Segment::Kanji(next))
            }
            Char::Kana => {
                let next = if let Some(idx) = self.rest.find(utils::is_kanji) {
                    let next = &self.rest[..idx];
                    self.rest = &self.rest[idx..];
                    next
                } else {
                    let next = self.rest;
                    self.rest = "";
                    next
                };
                Some(Segment::Kana(next))
            }
            Char::Exception => {
                let idx = next.len_utf8();
                let next = &self.rest[..idx];
                self.rest = &self.rest[idx..];
                Some(Segment::Exception(next))
            }
            Char::Other => {
                let idx = next.len_utf8();
                let next = &self.rest[..idx];
                self.rest = &self.rest[idx..];
                Some(Segment::Other(next))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Char {
    Kanji,
    Kana,
    Alphanumeric,
    Exception,
    Other,
}

fn classify_char(c: char) -> Char {
    if c == 'ヶ' {
        Char::Exception
    } else if utils::is_kanji(c) {
        Char::Kanji
    } else if utils::is_kana(c) {
        Char::Kana
    } else if utils::is_alphanumeric(c) {
        Char::Alphanumeric
    } else {
        Char::Other
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn segments_word() {
        let mut cs = CoarseSegmentation::new("物の怪");
        assert_eq!(Segment::Kanji("物"), cs.next().unwrap());
        assert_eq!(Segment::Kana("の"), cs.next().unwrap());
        assert_eq!(Segment::Kanji("怪"), cs.next().unwrap());
        assert!(cs.next().is_none());
    }

    #[test]
    fn segments_word_repeats() {
        let mut cs = CoarseSegmentation::new("物物のの怪怪のの");
        assert_eq!(Segment::Kanji("物物"), cs.next().unwrap());
        assert_eq!(Segment::Kana("のの"), cs.next().unwrap());
        assert_eq!(Segment::Kanji("怪怪"), cs.next().unwrap());
        assert_eq!(Segment::Kana("のの"), cs.next().unwrap());
        assert!(cs.next().is_none());
    }

    #[test]
    fn segments_word_kanji() {
        let mut fs = FineSegmentation::new("物の怪");
        assert_eq!(Segment::Kanji("物"), fs.next().unwrap());
        assert_eq!(Segment::Kana("の"), fs.next().unwrap());
        assert_eq!(Segment::Kanji("怪"), fs.next().unwrap());
        assert!(fs.next().is_none());
    }

    #[test]
    fn segments_word_kanji_repeats() {
        let mut fs = FineSegmentation::new("物物のの怪怪のの");
        assert_eq!(Segment::Kanji("物"), fs.next().unwrap());
        assert_eq!(Segment::Kanji("物"), fs.next().unwrap());
        assert_eq!(Segment::Kana("のの"), fs.next().unwrap());
        assert_eq!(Segment::Kanji("怪"), fs.next().unwrap());
        assert_eq!(Segment::Kanji("怪"), fs.next().unwrap());
        assert_eq!(Segment::Kana("のの"), fs.next().unwrap());
        assert!(fs.next().is_none());
    }

    #[test]
    fn segments_single_character_words() {
        let mut cs = CoarseSegmentation::new("一");
        assert_eq!(Segment::Kanji("一"), cs.next().unwrap());
        let mut fs = FineSegmentation::new("一");
        assert_eq!(Segment::Kanji("一"), fs.next().unwrap());
    }

    #[test]
    fn segments_mixed() {
        let mut cs = CoarseSegmentation::new("CDプレイヤー");
        assert_eq!(Segment::Alphanumeric("C"), cs.next().unwrap());
        assert_eq!(Segment::Alphanumeric("D"), cs.next().unwrap());
        assert_eq!(Segment::Kana("プレイヤー"), cs.next().unwrap());
        let mut fs = FineSegmentation::new("CDプレイヤー");
        assert_eq!(Segment::Alphanumeric("C"), fs.next().unwrap());
        assert_eq!(Segment::Alphanumeric("D"), fs.next().unwrap());
        assert_eq!(Segment::Kana("プレイヤー"), fs.next().unwrap());
    }
}
