//! Hungarian spelling: the Magyar Ispell Hunspell dictionary (`assets/proofing/hu-hu.*`, MPL-2.0,
//! see `README-hu-hu.txt` there) checked by the `spellbook` crate. Hungarian builds words from a
//! stem, many suffixes and compounds (`szövegszerkesztőprogramjainkban`), so a plain word list
//! can't check it; the affix rules and compounding of the dictionary do.
//!
//! The dictionary is inflated and parsed on first use (~0.1 s) and kept for the process.
//! `spellbook` is third-party code checked against hostile input here: words are length-capped,
//! suggestions are asked for the lower-case word (the library's re-capitalisation of suggestions
//! for capitalised words could panic on an empty suggestion) and recased here, and every call
//! runs under `catch_unwind`, a panic counting as "correct" with no suggestions.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;

/// Longest word (in characters) that is checked; longer ones are accepted unchecked.
const MAX_CHECK: usize = 80;
/// Longest word (in characters) that gets suggestions.
const MAX_SUGGEST: usize = 40;

static AFF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/hu-hu.aff.deflate"));
static DIC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/hu-hu.dic.deflate"));

fn dictionary() -> Option<&'static spellbook::Dictionary> {
    static D: OnceLock<Option<spellbook::Dictionary>> = OnceLock::new();
    D.get_or_init(|| match load() {
        Ok(d) => Some(d),
        Err(e) => {
            log::warn!("Hungarian spelling is unavailable: {e}");
            None
        }
    })
    .as_ref()
}

fn inflate(packed: &[u8], what: &str) -> Result<String, String> {
    if packed.is_empty() {
        return Err(format!("{what} was not built in"));
    }
    // The inflated files are ~2.3 MB and ~1.8 MB; the limit only guards against a corrupt build.
    let bytes = miniz_oxide::inflate::decompress_to_vec_with_limit(packed, 64 << 20).map_err(|e| format!("{what}: {e:?}"))?;
    String::from_utf8(bytes).map_err(|e| format!("{what}: {e}"))
}

fn load() -> Result<spellbook::Dictionary, String> {
    let aff = inflate(AFF, "hu-hu.aff")?;
    let dic = inflate(DIC, "hu-hu.dic")?;
    catch_unwind(|| spellbook::Dictionary::new(&aff, &dic))
        .map_err(|_| "the dictionary parser failed".to_string())?
        .map_err(|e| format!("hu-hu dictionary: {e}"))
}

/// Whether the Hungarian dictionary is built in and loads.
pub fn available() -> bool {
    dictionary().is_some()
}

/// Is `word` a correctly spelled Hungarian word? `None` when the dictionary is unavailable.
/// The caller has already accepted numbers, single letters, acronyms and user words.
pub fn check(word: &str) -> Option<bool> {
    let d = dictionary()?;
    if word.chars().count() > MAX_CHECK {
        return Some(true);
    }
    let w = word.replace('’', "'");
    Some(catch_unwind(AssertUnwindSafe(|| d.check(&w))).unwrap_or_else(|_| {
        log::warn!("Hungarian spell check failed on a {}-character word; accepting it", w.chars().count());
        true
    }))
}

/// Suggestions for a misspelled Hungarian word, best first, at most `max`, in the word's casing.
pub fn suggest(word: &str, max: usize) -> Vec<String> {
    let Some(d) = dictionary() else { return Vec::new() };
    let n = word.chars().count();
    if n == 0 || n > MAX_SUGGEST || max == 0 {
        return Vec::new();
    }
    let lower = word.replace('’', "'").to_lowercase();
    let mut found = Vec::new();
    if catch_unwind(AssertUnwindSafe(|| d.suggest(&lower, &mut found))).is_err() {
        log::warn!("Hungarian suggestions failed on a {n}-character word");
        return Vec::new();
    }
    let upper = n > 1 && word.chars().all(|c| !c.is_lowercase());
    let cap = word.chars().next().is_some_and(char::is_uppercase);
    let mut out: Vec<String> = Vec::new();
    for s in found {
        let s = s.trim();
        if s.is_empty() {
            continue;
        }
        let s = if upper {
            s.to_uppercase()
        } else if cap {
            let mut c = s.chars();
            c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
        } else {
            s.to_string()
        };
        if !out.contains(&s) {
            out.push(s);
        }
        if out.len() == max {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dictionary_is_built_in() {
        assert!(available());
    }

    #[test]
    fn inflected_and_compound_words_check() {
        for w in [
            "alma",
            "almát",
            "almáimmal",
            "házakban",
            "szövegszerkesztő",
            "helyesírás-ellenőrző",
            "Budapesten",
            "Magyarország",
            "körtefa",
            "egészségére",
            "legeslegmegszentségteleníthetetlenebbeknek",
            "megcsinálni",
            "fordítás",
            "ELLENŐRZŐ",
            "Egyébként",
        ] {
            assert_eq!(check(w), Some(true), "{w}");
        }
        for w in ["almmát", "hazakban", "szoveg", "ellenörző", "kövács", "nagyonnagyon"] {
            assert_eq!(check(w), Some(false), "{w}");
        }
    }

    #[test]
    fn suggestions_keep_the_words_casing() {
        assert_eq!(suggest("szoveg", 5).first().map(String::as_str), Some("szóvég"));
        assert!(suggest("szoveg", 5).contains(&"szöveg".to_string()));
        assert!(suggest("hazakban", 5).contains(&"házakban".to_string()));
        assert!(suggest("Ellenörző", 5).contains(&"Ellenőrző".to_string()), "{:?}", suggest("Ellenörző", 5));
        assert!(suggest("ELLENÖRZŐ", 5).contains(&"ELLENŐRZŐ".to_string()), "{:?}", suggest("ELLENÖRZŐ", 5));
        assert!(suggest("szoveg", 2).len() <= 2);
        assert!(suggest(&"á".repeat(41), 5).is_empty(), "too long for suggestions");
    }

    /// Hostile input never panics (the library is third-party): odd letters, combining marks,
    /// ligatures, separators, very long words, and capitalised words, whose suggestions once
    /// crashed the library.
    #[test]
    fn hostile_words_never_panic() {
        let alphabet: Vec<char> = "aábcdeéfgiíjklmnoóöőpqrstuúüűvxyzAÁÉŐŰÖÜ-.'’0123456789ccsggyllynnysszttyszerű\u{301}ß€‰§%°ﬁ ".chars().collect();
        let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        for i in 0..3000 {
            let len = (next() % 30) as usize + 1;
            let w: String = (0..len).filter_map(|_| alphabet.get((next() as usize) % alphabet.len())).collect();
            let _ = check(&w);
            if i % 25 == 0 {
                let _ = suggest(&w, 5);
                let mut cap = w.chars();
                let capped: String = cap.next().map(|f| f.to_uppercase().chain(cap).collect()).unwrap_or_default();
                let _ = suggest(&capped, 5);
            }
        }
        for w in ["", " ", "-", "'", "’", "Ł", "ﬁ", "Kovácscsal", "Ccs", "ÁÉÍÓÖŐÚÜŰ", &"ő".repeat(500), &"Ab".repeat(30)] {
            let _ = check(w);
            let _ = suggest(w, 5);
        }
    }
}
