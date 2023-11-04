use std::fmt::{self, Display, Formatter};

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Copy, Clone, serde::Deserialize, serde::Serialize,
)]
pub enum Challenges {
    #[default]
    C2331,
    C2332,
    C2333,
    None,
}

impl Challenges {
    pub fn iter() -> impl Iterator<Item = Self> {
        use Challenges::*;
        [C2331, C2332, C2333].iter().copied()
    }

    fn _next(&self) -> Self {
        match self {
            Challenges::C2331 => Challenges::C2332,
            Challenges::C2332 => Challenges::C2333,
            Challenges::C2333 => Challenges::C2331,
            Challenges::None => Challenges::None,
        }
    }

    pub fn get_info_url(&self) -> String {
        match self {
            Challenges::C2331 => "https://raw.githubusercontent.com/bitbrain-za/judge_2331-rs/main/src/generator/2331.md".to_string(),
            Challenges::C2332 => "https://raw.githubusercontent.com/bitbrain-za/judge_2331-rs/main/src/generator/2332.md".to_string(),
            Challenges::C2333 => "https://raw.githubusercontent.com/bitbrain-za/judge_2331-rs/main/src/generator/2333.md".to_string(),
            Challenges::None => "".to_string(),
        }
    }
}

impl Display for Challenges {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Challenges::C2331 => write!(f, "23_3_1"),
            Challenges::C2332 => write!(f, "23_3_2"),
            Challenges::C2333 => write!(f, "23_3_3"),
            Challenges::None => write!(f, "None"),
        }
    }
}
