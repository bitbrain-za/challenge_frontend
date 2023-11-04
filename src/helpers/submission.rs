use web_sys::FormData;

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
            log::info!("Binary: {:?}", binary);
            let uint8array = js_sys::Uint8Array::from(binary.as_slice());
            let array = js_sys::Array::from(&uint8array);
            let blob = web_sys::Blob::new_with_u8_array_sequence(array.as_ref()).unwrap();
            form.append_with_blob("binary", &blob).unwrap();
        }

        log::info!("Form: {:?}", form);

        form
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SubmissionResult {
    Success { score: u32, message: String },
    Failure { message: String },
}
