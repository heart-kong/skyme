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
