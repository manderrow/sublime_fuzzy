//! Fuzzy matching algorithm based on Sublime Text's string search. Iterates through
//! characters of a search string and calculates a score.
//!
//! The score is based on several factors:
//! * **Word starts** like the `t` in `some_thing` get a bonus (`bonus_word_start`)
//! * **Consecutive matches** get an accumulative bonus for every consecutive match (`bonus_consecutive`)
//! * Matches that also match **case** (`T` -> `T` instead of `t` -> `T`) in case of a case-insensitive search get a bonus (`bonus_match_case`)
//! * The **distance** between two matches will be multiplied with the `penalty_distance` penalty and subtracted from the score
//!
//! The default scoring is configured to give a lot of weight to word starts. So a pattern `scc` will match
//! **S**occer**C**artoon**C**ontroller, not **S**o**cc**erCartoonController.
//!
//! # Match Examples
//!
//! With default weighting.
//!
//! | Pattern       | Target string             | Result
//! | ---           | ---                       | ---
//! | `scc`         | `SoccerCartoonController` | **S**occer**C**artoon**C**ontroller
//! | `something`   | `some search thing`       | **some** search **thing**
//!
//! # Usage
//!
//! Basic usage:
//!
//! ```rust
//! use sublime_fuzzy::best_match;
//!
//! let result = best_match("something", "some search thing");
//!
//! assert!(result.is_some());
//! ```
//!
//! [`Match::continuous_matches`] returns an iter of consecutive matches. Based on those the input
//! string can be formatted.
//!
//! [`format_simple`] provides a simple formatting that wraps matches in tags:
//!
//! ```rust
//! use sublime_fuzzy::{best_match, format_simple};
//!
//! let target = "some search thing";
//!
//! let result = best_match("something", target).unwrap();
//!
//! assert_eq!(
//!     format_simple(&result, target, "<span>", "</span>"),
//!     "<span>some</span> search <span>thing</span>"
//! );
//! ```
//!
//! The weighting of the different factors can be adjusted:
//!
//! ```rust
//! use sublime_fuzzy::{FuzzySearch, Scoring};
//!
//! // Or pick from one of the provided `Scoring::...` methods like `emphasize_word_starts`
//! let scoring = Scoring {
//!     bonus_consecutive: 128,
//!     bonus_word_start: 0,
//!     ..Scoring::default()
//! };
//!
//! let result = FuzzySearch::new("something", "some search thing")
//!     .score_with(&scoring)
//!     .best_match();
//!
//! assert!(result.is_some())
//! ```
//!
//! **Note:** Any whitespace in the pattern (`'something'`
//! in the examples above) will be removed.
//!

mod matching;
mod parsing;
mod scoring;
mod search;

pub use matching::{ContinuousMatch, ContinuousMatches, Match};
pub use parsing::Query;
pub use scoring::Scoring;
pub use search::best_match;

/// Formats a [`Match`] by appending `before` before any matches and `after`
/// after any matches.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use sublime_fuzzy::{best_match, format_simple};
///
/// let target_string = "some search thing";
/// let result = best_match("something", target_string).unwrap();
///
/// assert_eq!(
///     format_simple(&result, target_string, "<span>", "</span>"),
///     "<span>some</span> search <span>thing</span>"
/// );
/// ```
///
pub fn format_simple(match_: &Match, target: &str, before: &str, after: &str) -> String {
    let str_before = before.to_owned();
    let str_after = after.to_owned();

    let mut pieces = Vec::new();

    let mut last_end: u32 = 0;

    for c in match_.continuous_matches() {
        // Piece between last match and this match
        pieces.push(
            target
                .chars()
                .skip(last_end as usize)
                .take((c.start() - last_end) as usize)
                .collect::<String>(),
        );

        pieces.push(str_before.clone());

        // This match
        pieces.push(
            target
                .chars()
                .skip(c.start() as usize)
                .take(c.len() as usize)
                .collect(),
        );

        pieces.push(str_after.clone());

        last_end = c.start() + c.len();
    }

    // Leftover chars
    if last_end as usize != target.len() {
        pieces.push(target.chars().skip(last_end as usize).collect::<String>());
    }

    pieces.join("")
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;

    use crate::{Match, Scoring, format_simple, matching::ContinuousMatch, parsing::Query};

    fn best_match<'bump>(
        bump: &'bump Bump,
        query: &str,
        haystack: &str,
    ) -> Option<&'bump Match<'bump>> {
        super::best_match(
            bump,
            &Query::new(bump, query),
            &Scoring::default(),
            haystack,
        )
    }

    #[test]
    fn feature_serde() {
        assert!(cfg!(feature = "serde"));
    }

    #[test]
    fn full_match() {
        let bump = Bump::new();
        assert!(best_match(&bump, "test", "test").is_some());
    }

    #[test]
    fn any_match() {
        let bump = Bump::new();
        assert!(best_match(&bump, "towers", "the two towers").is_some());
    }

    #[test]
    fn no_match() {
        let bump = Bump::new();
        assert_eq!(best_match(&bump, "abc", "def"), None);
    }

    #[test]
    fn basic() {
        let bump = Bump::new();
        let r = best_match(&bump, "scc", "soccer cartoon controller");

        assert!(r.is_some());
    }

    #[test]
    fn partial_match_none() {
        let bump = Bump::new();
        assert_eq!(best_match(&bump, "partial", "part"), None);
    }

    #[test]
    fn case_insensitivity() {
        let bump = Bump::new();
        assert!(
            best_match(&bump, "ttt", "The Two Towers").is_some(),
            "Lower query chars do not match upper target chars"
        );

        assert!(
            best_match(&bump, "TTT", "The Two Towers").is_some(),
            "Upper query chars do not match upper target chars"
        );

        assert!(
            best_match(&bump, "TTT", "the two towers").is_some(),
            "Upper query chars do not match lower target chars"
        );
    }

    #[test]
    fn case_insensitive_scoring() {
        let bump = Bump::new();
        let non_case_match = best_match(&bump, "ttt", "The Two Towers").unwrap();
        let case_match = best_match(&bump, "TTT", "The Two Towers").unwrap();

        assert!(non_case_match.score() == case_match.score());
    }

    #[test]
    fn whitespace() {
        let bump = Bump::new();
        assert!(best_match(&bump, "t t", "The Two Towers").is_some());
    }

    #[test]
    fn word_starts_count_more() {
        let bump = Bump::new();
        let r = best_match(&bump, "something", "some search thing");

        assert_eq!(
            r.unwrap()
                .continuous_matches()
                .collect::<Vec<ContinuousMatch>>(),
            vec![ContinuousMatch::new(0, 4), ContinuousMatch::new(12, 5)]
        );
    }

    #[test]
    fn word_starts_count_more_2() {
        let bump = Bump::new();
        let m = best_match(&bump, "scc", "SccsCoolController").unwrap();

        assert_eq!(
            m.continuous_matches().collect::<Vec<ContinuousMatch>>(),
            vec![ContinuousMatch::new(0, 3)]
        );
    }

    #[test]
    fn empty_query() {
        let bump = Bump::new();
        assert_eq!(best_match(&bump, "", "test"), None);
    }

    #[test]
    fn empty_target() {
        let bump = Bump::new();
        assert_eq!(best_match(&bump, "test", ""), None);
    }

    #[test]
    fn distance_to_first_is_ignored() {
        let bump = Bump::new();
        let a = best_match(&bump, "release", "some_release").unwrap();
        let b = best_match(&bump, "release", "a_release").unwrap();

        assert_eq!(a.score(), b.score());
    }

    #[test]
    fn matches_unicode() {
        let bump = Bump::new();
        let m = best_match(&bump, "👀", "🦀 👈 👀").unwrap();

        assert_eq!(m.matched_indices().copied().collect::<Vec<_>>(), vec![4]);
    }

    #[test]
    fn formats_unicode() {
        let s = "🦀 👈 👀";
        let bump = Bump::new();
        let m = best_match(&bump, "👀", s).unwrap();

        assert_eq!(format_simple(&m, s, "<", ">"), "🦀 👈 <👀>");
    }
}
