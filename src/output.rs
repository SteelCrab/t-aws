use std::fs;
use std::io;

pub fn save_markdown(filename: &str, content: &str) -> io::Result<()> {
    fs::write(filename, content)
}

#[cfg(test)]
mod tests {
    use super::save_markdown;
    use std::fs;

    #[test]
    fn save_markdown_writes_expected_content() {
        let mut path = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        path.push(format!(
            "emd-output-test-{}-{}.md",
            std::process::id(),
            nanos
        ));

        save_markdown(path.to_str().unwrap_or("output.md"), "# title\ncontent\n")
            .expect("save markdown");
        let written = fs::read_to_string(&path).expect("read written markdown");
        assert_eq!(written, "# title\ncontent\n");

        let _ = fs::remove_file(&path);
    }
}
