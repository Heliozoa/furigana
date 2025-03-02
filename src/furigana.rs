use crate::{reading_equivalent, segmentation::Segment};
use std::fmt::Display;

/// A mapping of furigana to a word.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Furigana<'a> {
    /// The original word with furigana.
    pub furigana: Vec<FuriganaSegment<'a>>,
    /// A rough relative measure of this mapping's accuracy, the higher the more accurate.
    /// Only meaningful in comparison with other configurations for the same word.
    pub accuracy: i32,
}

impl<'a> Furigana<'a> {
    /// Flattens `FuriganaNode`s to a list of `Furigana`.
    pub fn from_tree(tree: &FuriganaTree<'a>) -> Vec<Self> {
        Self::from_tree_inner(&tree.nodes)
    }

    fn from_tree_inner(nodes: &[FuriganaNode<'a>]) -> Vec<Self> {
        let mut furigana: Vec<Furigana> = vec![];

        for node in nodes {
            let word = node.segment.inner();
            let reading = if reading_equivalent(word, node.reading) {
                // no need for furigana here
                None
            } else {
                Some(node.reading)
            };
            let kanji_accuracy = match &node.kanji_accurate {
                Some(KanjiAccuracy::Accurate) => 2,
                Some(KanjiAccuracy::AccurateWithRendaku) => 1,
                Some(KanjiAccuracy::AccurateWithSokuonbin) => 1,
                Some(KanjiAccuracy::Inaccurate) => -2,
                None => 0,
            };
            if node.extensions.is_empty() {
                furigana.push(Furigana {
                    furigana: vec![FuriganaSegment {
                        segment: word,
                        furigana: reading,
                    }],
                    accuracy: kanji_accuracy,
                });
            } else {
                for flattened_extensions in Self::from_tree_inner(&node.extensions) {
                    let mut ruby = vec![FuriganaSegment {
                        segment: word,
                        furigana: reading,
                    }];
                    ruby.extend(flattened_extensions.furigana);
                    let kanji_accuracy = kanji_accuracy + flattened_extensions.accuracy;
                    furigana.push(Furigana {
                        furigana: ruby,
                        accuracy: kanji_accuracy,
                    })
                }
            }
        }
        furigana
    }
}

/// Prints the word with its furigana using HTML ruby tags.
impl Display for Furigana<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<ruby>")?;
        for segment in &self.furigana {
            write!(f, "{}", segment.segment)?;
            write!(f, "<rt>")?;
            if let Some(furigana) = segment.furigana {
                write!(f, "{}", furigana)?;
            }
            write!(f, "</rt>")?;
        }
        write!(f, "</ruby>")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuriganaSegment<'a> {
    /// A segment of the original word.
    pub segment: &'a str,
    /// The furigana corresponding to the segment, if any.
    pub furigana: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuriganaTree<'a> {
    pub text: &'a str,
    pub reading: &'a str,
    pub nodes: Vec<FuriganaNode<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuriganaNode<'a> {
    /// The corresponding segment from the original word.
    pub segment: Segment<'a>,
    /// The reading of the segment.
    pub reading: &'a str,
    /// Possible ways to continue after this point.
    pub extensions: Vec<FuriganaNode<'a>>,
    /// The accuracy of this reading according to known kanji readings. None when inapplicable, such as for kana segments.
    pub kanji_accurate: Option<KanjiAccuracy>,
}

/// The accuracy of a given reading for a kanji.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KanjiAccuracy {
    Accurate,
    AccurateWithRendaku,
    AccurateWithSokuonbin,
    Inaccurate,
}
