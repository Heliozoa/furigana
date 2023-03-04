# furigana

[![Crates.io](https://img.shields.io/crates/v/furigana)](https://crates.io/crates/furigana)
[![docs.rs](https://img.shields.io/badge/docs.rs-furigana-success)](https://docs.rs/furigana)
[![Crates.io](https://img.shields.io/crates/l/furigana)](https://choosealicense.com/licenses/mpl-2.0/)
[![GitHub](https://img.shields.io/badge/GitHub-Heliozoa-24292f)](https://github.com/Heliozoa/furigana)

Contains functionality for correctly mapping furigana to a word given a reading, and optionally kanji reading data.

## Usage

```rs
for mapping in furigana::map_naive("物の怪", "もののけ") {
    println!("{mapping}");
}
```

prints out the following mappings:

<pre>
<ruby>物<rt>も</rt>の<rt></rt>怪<rt>のけ</rt></ruby>

<ruby>物<rt>もの</rt>の<rt></rt>怪<rt>け</rt></ruby>
</pre>

Only the first reading is actually valid, but based only on a word and its reading there's no way to know that.

If given information about kanji readings (for example, from [KANJIDIC2](http://www.edrdg.org/kanjidic/)), `furigana::map` is able to grade the potential mappings:

```rs
let mut kanji_to_readings = HashMap::new();
kanji_to_readings.insert("物".to_string(), vec!["もの".to_string()]);
kanji_to_readings.insert("怪".to_string(), vec!["け".to_string()]);
let mapping = furigana::map("物の怪", "もののけ", &kanji_to_readings)
    .into_iter()
    .max_by_key(|f| f.accuracy)
    .unwrap();
println!("{mapping}");
```

assigns a high accuracy value to the correct reading and a low value to the other one, so that only the correct mapping is printed:

<pre>
<ruby>物<rt>もの</rt>の<rt></rt>怪<rt>け</rt></ruby>
</pre>

## Notes

- The algorithm used is recursive and not optimised, so it may be inefficient for long, kanji-heavy inputs.

- If the library fails to produce the correct mapping, or if its accuracy is lower than an incorrect mapping's, a GitHub issue is much appreciated!

## License

Licensed under the Mozilla Public License Version 2.0.
