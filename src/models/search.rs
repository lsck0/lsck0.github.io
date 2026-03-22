use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};

pub fn fuzzy_score(query: &str, text: &str) -> Option<u32> {
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
    let pattern = Pattern::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        nucleo_matcher::pattern::AtomKind::Fuzzy,
    );
    let mut buf = Vec::new();
    let haystack = Utf32Str::new(text, &mut buf);
    return pattern.score(haystack, &mut matcher);
}
