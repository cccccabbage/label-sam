use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub yolo_path: String,
    pub sam_e_path: String,
    pub sam_d_path: String,
}

impl Config {
    pub fn new() -> Config {
        let json_content = std::fs::read_to_string("config.json").unwrap();

        serde_json::from_str(&json_content).unwrap()
    }
}
