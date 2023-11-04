#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Submission {
    pub player: String,
    pub challenge: String,
    pub filename: String,
    pub language: String,
    pub test: bool,

    pub code: String,
    #[serde(skip)]
    pub binary: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SubmissionResult {
    Success { score: u32, message: String },
    Failure { message: String },
}
