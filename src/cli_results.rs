use std::collections::HashMap;
use std::fmt::Display;

use serde::export::Formatter;

use crate::application::RepositoryUrlDto;

#[derive(Debug)]
pub struct CountBranchesResult {
    hash_map: HashMap<RepositoryUrlDto, u32>,
}

impl Display for CountBranchesResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut lines: Vec<String> = self
            .hash_map
            .iter()
            .map(|(url, count)| format!("{}: {}", url, count))
            .collect::<Vec<String>>();
        lines.sort();

        write!(f, "# Repository Branch Counts\n\n{}", lines.join("\n"))
    }
}

impl From<HashMap<RepositoryUrlDto, u32>> for CountBranchesResult {
    fn from(hash_map: HashMap<RepositoryUrlDto, u32>) -> Self {
        CountBranchesResult { hash_map }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use spectral::prelude::*;

    use super::*;

    fn count_branches_result<S: AsRef<str>>(hash_map: HashMap<S, u32>) -> CountBranchesResult {
        CountBranchesResult {
            hash_map: HashMap::from_iter(
                hash_map
                    .iter()
                    .map(|(url, &count)| (RepositoryUrlDto::new(url.as_ref().to_string()), count)),
            ),
        }
    }

    #[test]
    fn implements_display() {
        let mut hash_map = HashMap::new();
        hash_map.insert("url", 1);
        let under_test = count_branches_result(hash_map);
        assert_that(&under_test.to_string())
            .is_equal_to(&"# Repository Branch Counts\n\nurl: 1".to_string());
    }

    #[test]
    fn implements_display_and_sorts_line_order() {
        let mut hash_map = HashMap::new();
        hash_map.insert("url", 1);
        hash_map.insert("other_url", 0);
        let under_test = count_branches_result(hash_map);
        assert_that(&under_test.to_string())
            .is_equal_to(&"# Repository Branch Counts\n\nother_url: 0\nurl: 1".to_string());
    }
}
