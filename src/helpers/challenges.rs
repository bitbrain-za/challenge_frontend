use crate::helpers::{fetchers::Requestor, AppState};
use std::fmt::{self, Display, Formatter};
use std::sync::{Arc, Mutex};

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Copy, Clone, serde::Deserialize, serde::Serialize,
)]
pub enum Challenges {
    C2331,
    #[default]
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
}

impl Display for Challenges {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Challenges::C2331 => write!(f, "2331"),
            Challenges::C2332 => write!(f, "2332"),
            Challenges::C2333 => write!(f, "2333"),
            Challenges::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Challenge {
    pub name: String,
    pub command: String,
    pub table: String,
    doc: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChallengeCollection {
    pub items: Vec<Challenge>,
    url: String,
}

impl Default for ChallengeCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl ChallengeCollection {
    pub fn new() -> Self {
        let url = option_env!("BACKEND_URL")
            .unwrap_or("http://123.4.5.6:3000/")
            .to_string();

        Self {
            items: Vec::new(),
            url,
        }
    }

    pub fn fetch(&mut self, app_state: Arc<Mutex<AppState>>) -> Option<Requestor> {
        let url = format!("{}api/game/challenge", self.url);

        log::debug!("Fetching challenge info");
        let app_state = Arc::clone(&app_state.clone());
        let mut getter = Requestor::new_get(app_state, &url, true);
        getter.send();
        Some(getter)
    }

    pub fn from_json(json: &str) -> Self {
        let items: Vec<Challenge> = serde_json::from_str(json).unwrap_or_default();
        log::debug!("Found {} challenges", items.len());
        Self {
            items,
            ..Default::default()
        }
    }

    pub fn get_instructions(&self, challenge: Challenges) -> String {
        log::debug!("Getting instructions for {}", challenge);
        match self
            .items
            .iter()
            .find(|c| c.command == challenge.to_string())
        {
            Some(c) => c.doc.clone(),
            None => String::from("No instructions found"),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn test_parse_json() {
        let output = r#"[
  {
    "command": "2331",
    "doc": " # Find Odds\n\n## Problem\n\nGiven an array of integers, find the one that appears an odd number of times.\n\nThere will always be only one integer that appears an odd number of times.\n\nExamples:\n\n- [7] should return 7, because it occurs 1 time (which is odd).\n- [0] should return 0, because it occurs 1 time (which is odd).\n- [1,1,2] should return 2, because it occurs 1 time (which is odd).\n- [0,1,0,1,0] should return 0, because it occurs 3 times (which is odd).\n- [1,2,2,3,3,3,4,3,3,3,2,2,1] should return 4, because it appears 1 time (which is odd).\n\n## Instructions\n\nDesign a program/script that can solve this problem in a fast/interesting/elegant way.\nYour program should be able to accept a single command line argument for a filename.\n\nThis file will be a json formatted array of arrays of numbers ranging from 0-255.\n\nReturn your results as a single array.\nFor example If you ran a file with the 5 examples above the output would look like: [7,0,2,3,4]\n\n## Testing:\n\nThere is a JSON sample file containing 500 arrays for you to view and test against.\nThere is a script called \"check.sh\" which you can use to check against the sample data.\n\nTo use the script, call it with the command to run your program as an argument\n\n```bash\n./check.sh \"./my_program\"\n```\n\n## Making an attempt\n\nFor a real run, you will be tested against 100 000 random samples.\n\n```bash\njudge -C 2331 -n Player1 -c \"python my_code.py\" -L python\njudge -C 2331 -n Player2 -c \"./my_binary\" -L go\n```\n\n",
    "name": "Find the odd one out",
    "table": "23_3_1"
  },
  {
    "command": "2332",
    "doc": " # Find Odd One Out Two\n\n## Problem\n\nGiven an array of integers, find the one that appears an odd number of times.\n\nThere will always be only one integer that appears an odd number of times.\n\nExamples:\n\n- [7] should return 7, because it occurs 1 time (which is odd).\n- [0] should return 0, because it occurs 1 time (which is odd).\n- [1,1,2] should return 2, because it occurs 1 time (which is odd).\n- [0,1,0,1,0] should return 0, because it occurs 3 times (which is odd).\n- [1,2,2,3,3,3,4,3,3,3,2,2,1] should return 4, because it appears 1 time (which is odd).\n\n## Instructions\n\nIf the arguments are empty, print a \"0\" and return.\n\nDesign a program/script that can solve this problem in a fast/interesting/elegant way.\n\nYour program will need to run in a loop, listing to stdin.\nAn array will be given in the form \"5,7,2,7,2,3,5\\n\" and you will output \"3\\n\".\n\nNB: inputs and outputs must all be terminated with a newline!\nIf your program receives a \"q\\n\" as an input, it must exit gracefully and quietly.\n\n## Making an attempt\n\nFor a real run, you will be tested against 10 000 random samples.\n\n```bash\njudge -C 2332 -c \"python my_code.py\" -L python\njudge -C 2332 -c \"./my_binary\" -L go\n```\n\n",
    "name": "Find the odd one out two",
    "table": "23_3_2"
  },
  {
    "command": "2333",
    "doc": " # How Big?!\n\n## Problem\n\nGiven an array of integers, find the magnitude.\nReturn your result as an unsigned integer (discard the non-integer components)\n\nExamples:\n\n- [7] should return 7\n- [0] should return 0\n- [1,1,2] should return 2\n- [1,2,3,4] should return 5\n- [5,5,5,5] should return 10\n\n## Instructions\n\nIf the arguments are empty, print a \"0\" and return.\n\nDesign a program/script that can solve this problem in a fast/interesting/elegant way.\n\nYour program will need to run in a loop, listing to stdin.\nAn array will be given in the form \"5,7,2,7,2,3,5\\n\" and you will output \"13\\n\".\n\nNB: inputs and outputs must all be terminated with a newline!\nIf your program receives a \"q\\n\" as an input, it must exit gracefully and quietly.\n\n## Making an attempt\n\nFor a real run, you will be tested against 10 000 random samples.\n\n```bash\njudge -C 2333 -c \"python my_code.py\" -L python\njudge -C 2333 \"./my_binary\" -L go\n```\n\n",
    "name": "How big?",
    "table": "23_3_3"
  },
  {
    "command": "2334",
    "doc": " # Find the bad character\n\n## Problem\n\nGiven a string, find the non alphanumeric character. Anything outside of the A-Z, a-z and 0-9 range\nReturn the bad character. If more than one bad character are in the string, just return the first one.\n\nExamples:\n\n- \"Hello,world\" should return ,\n- \"This|isbad\" should return |\n- \"123456=89\" should return =\n\n## Instructions\n\nDesign a program/script that can solve this problem in a fast/interesting/elegant way.\n\nYour program will need to run in a loop, listing to stdin.\nA string will be given in the form \"sting_to_be_inspected\\n\" and you will output \"_\\n\".\n\nNB: inputs and outputs must all be terminated with a newline!\nIf your program receives a \"\\n\" as an input, it must exit gracefully and quietly.\n\n## Making an attempt\n\nFor a real run, you will be tested against X random samples.\n\n```bash\njudge -C 2333 -c \"python my_code.py\" -L python\njudge -C 2333 \"./my_binary\" -L go\n```\n\n\n",
    "name": "Input validation",
    "table": "23_3_4"
  }
]"#;

        let _: Value = serde_json::from_str(output).unwrap();
        let challenges = ChallengeCollection::from_json(output);

        assert_eq!(challenges.items[0].name, "Find the odd one out");
    }
}
