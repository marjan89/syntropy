use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

#[derive(Default)]
pub struct FuzzySearcher {
    matcher: SkimMatcherV2,
}

impl FuzzySearcher {
    pub fn search<T>(&self, items: &[T], query: &str) -> Vec<usize>
    where
        T: std::ops::Deref,
        T::Target: AsRef<str>,
    {
        if query.is_empty() {
            return (0..items.len()).collect();
        }

        let mut matches: Vec<_> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                self.matcher
                    .fuzzy_match(item.deref().as_ref(), query)
                    .map(|score| (idx, score))
            })
            .collect();

        matches
            .sort_by(|(_lhs_index, lhs_score), (_rhs_index, rhs_score)| rhs_score.cmp(lhs_score));

        matches.into_iter().map(|(idx, _)| idx).collect()
    }
}
