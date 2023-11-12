use super::{
    fetchers::{RequestStatus, Requestor},
    AppState, Challenges, Languages,
};
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use web_sys::FormData;

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

impl Submission {
    pub fn to_formdata(&self) -> FormData {
        let form = FormData::new().unwrap();
        form.append_with_str("challenge", &self.challenge.to_string())
            .unwrap();
        form.append_with_str("filename", &self.filename).unwrap();
        form.append_with_str("language", &self.language.to_string())
            .unwrap();
        form.append_with_str("test", &self.test.to_string())
            .unwrap();
        if let Some(code) = &self.code {
            form.append_with_str("code", code).unwrap();
        }
        if let Some(binary) = &self.binary {
            let uint8arr =
                js_sys::Uint8Array::new(&unsafe { js_sys::Uint8Array::view(binary) }.into());
            let array = js_sys::Array::new();
            array.push(&uint8arr.buffer());
            let blob = web_sys::Blob::new_with_u8_array_sequence(array.as_ref()).unwrap();
            form.append_with_blob("binary", &blob).unwrap();
        }

        log::info!("Form: {:?}", form);

        form
    }

    pub fn check_sender(sender: &mut Option<Requestor>) -> SubmissionResult {
        if let Some(requestor) = sender {
            let result = &requestor.check_promise();

            match result {
                RequestStatus::Success(text) => {
                    *sender = None;
                    match serde_json::from_str::<SubmissionResult>(text) {
                        Ok(submission_response) => submission_response.clone(),
                        Err(error) => SubmissionResult::Failure {
                            message: error.to_string(),
                        },
                    }
                }
                RequestStatus::Failed(e) => {
                    *sender = None;
                    SubmissionResult::Failure {
                        message: e.to_string(),
                    }
                }
                RequestStatus::InProgress => SubmissionResult::Busy,
                RequestStatus::NotStarted => SubmissionResult::NotStarted,
            }
        } else {
            SubmissionResult::NotStarted
        }
    }

    pub fn sender(&self, app_state: Arc<Mutex<AppState>>, url: &str) -> Option<Requestor> {
        let mut submitter = if self.code.is_some() {
            let submission = Some(serde_json::to_string(&self).unwrap());
            Requestor::new_post(app_state, url, true, submission)
        } else {
            let submission = Some(self.to_formdata());
            Requestor::new_form_post(app_state, url, true, submission)
        };
        submitter.send();
        Some(submitter)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.challenge == Challenges::None {
            return Err("Challenge not selected".to_string());
        }
        if self.filename.is_empty() {
            return Err("Filename not specified".to_string());
        }
        if self.code.is_none() && self.binary.is_none() {
            return Err("Code not specified".to_string());
        }

        let rx = regex::Regex::new(r"^[a-zA-Z0-9_\-\.]+$").unwrap();
        if !rx.is_match(&self.filename) {
            return Err("Filename contains invalid characters".to_string());
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SubmissionResult {
    #[default]
    NotStarted,
    Success {
        score: u32,
        message: String,
    },
    Failure {
        message: String,
    },
    NotAuthorized,
    Busy,
}

impl Display for SubmissionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmissionResult::NotStarted => write!(f, ""),
            SubmissionResult::Success { score: _, message } => {
                write!(f, "{}", message)
            }
            SubmissionResult::Failure { message } => write!(f, "Failure: {}", message),
            SubmissionResult::NotAuthorized => write!(f, "Not authorized"),
            SubmissionResult::Busy => write!(f, "Busy"),
        }
    }
}
