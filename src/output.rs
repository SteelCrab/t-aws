use std::fs;
use std::io;

pub fn save_markdown(filename: &str, content: &str) -> io::Result<()> {
    fs::write(filename, content)
}
