//! `furigana::map_naive` is convenient because it doesn't require any kanji reading information.
//! However, without such knowledge some word-reading pairs are ambiguous.

fn main() {
    for mapping in furigana::map_naive("物の怪", "もののけ") {
        println!("{mapping}");
    }
}
