use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::{fs::File, io::BufReader};

#[derive(Serialize, Deserialize)]
pub struct Token {
    token: String,
}

impl Token {
    pub fn get_token(file_name: &str) -> Result<String> {
        let file = File::open(file_name).unwrap();
        let reader = BufReader::new(file);
        let t: Token = serde_json::from_reader(reader).unwrap();
        Ok(t.token)
    }
}
