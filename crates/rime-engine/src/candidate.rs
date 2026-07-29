use skyme_common::Candidate;

/// A page of candidates returned by the Rime engine.
///
/// Wraps a vector of `Candidate` items with pagination metadata.
#[derive(Clone, Debug)]
pub struct CandidateList {
    pub candidates: Vec<Candidate>,
    pub page: u32,
    pub page_size: u32,
    pub is_last_page: bool,
}

impl CandidateList {
    pub fn new(
        candidates: Vec<Candidate>,
        page: u32,
        page_size: u32,
        is_last_page: bool,
    ) -> Self {
        Self {
            candidates,
            page,
            page_size,
            is_last_page,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::candidate::CandidateList;
    use skyme_common::Candidate;

    #[test]
    fn test_candidate_list_empty() {
        let list = CandidateList::new(vec![], 0, 5, true);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_candidate_list_with_items() {
        let candidates = vec![
            Candidate { text: "中".into(), comment: "zhong1".into(), index: 0, quality: 0.9 },
            Candidate { text: "国".into(), comment: "guo2".into(), index: 1, quality: 0.8 },
        ];
        let list = CandidateList::new(candidates.clone(), 0, 5, false);
        assert!(!list.is_empty());
        assert_eq!(list.len(), 2);
        assert!(!list.is_last_page);
        assert_eq!(list.page, 0);
        assert_eq!(list.page_size, 5);
    }

    #[test]
    fn test_candidate_list_page() {
        let list = CandidateList::new(vec![], 1, 5, true);
        assert_eq!(list.page, 1);
        assert_eq!(list.page_size, 5);
        assert!(list.is_last_page);
    }
}
