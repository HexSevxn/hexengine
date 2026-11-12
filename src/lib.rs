pub mod editor;
pub mod engine;
pub mod game;

pub fn read_file(path: &str) -> Vec<String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file = File::open(path).expect(&format!("File {} cannot be found.", path));
    let reader = BufReader::new(file);
    reader
        .lines()
        .into_iter()
        .map(|x| x.expect(&format!("Error reading lines of file: {}", path)))
        .collect::<Vec<String>>()
}
