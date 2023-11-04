use super::{Challenges, Languages};

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Submission {
    pub challenge: Challenges,
    pub filename: String,
    pub language: Languages,
    pub test: bool,

    pub code: Option<String>,
    #[serde(skip)]
    pub binary: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SubmissionResult {
    Success { score: u32, message: String },
    Failure { message: String },
}
