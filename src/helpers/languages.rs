use std::fmt::{self, Display, Formatter};

#[derive(Default, PartialEq, Eq, Hash, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum Languages {
    #[default]
    C,
    Cpp,
    CSharp,
    Go,
    Java,
    JavaScript,
    Python,
    Rust,
    ShellScript,
}

impl Languages {
    pub fn iter() -> impl Iterator<Item = Self> {
        use Languages::*;
        [
            C,
            Cpp,
            CSharp,
            Go,
            Java,
            JavaScript,
            Python,
            Rust,
            ShellScript,
        ]
        .iter()
        .copied()
    }
}

impl Display for Languages {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Languages::C => write!(f, "C"),
            Languages::Cpp => write!(f, "C++"),
            Languages::CSharp => write!(f, "C#"),
            Languages::Go => write!(f, "Go"),
            Languages::Java => write!(f, "Java"),
            Languages::JavaScript => write!(f, "JavaScript"),
            Languages::Python => write!(f, "Python"),
            Languages::Rust => write!(f, "Rust"),
            Languages::ShellScript => write!(f, "Bash"),
        }
    }
}
