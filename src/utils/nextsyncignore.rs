use std::fs::File;
use std::io::{BufReader, BufRead};
use regex::Regex;
use crate::utils::path;

pub fn read_lines() -> Result<Vec<String>, ()> {
    if let Some(path) = path::nextsyncignore() {
        let file = match File::open(path) {
            Ok(buffer) => buffer,
            Err(_) => return Err(()),
        };
        let reader = BufReader::new(file);
        let mut lines = vec![];
        for line in reader.lines() {
            if let Ok(l) = line {
                lines.push(normalize_rule(l.clone()));
            } else {
                return Err(());
            }
        }
        return Ok(lines);
    }
    Ok(vec![])
}

pub fn _ignore_files(files: &mut Vec<String>) -> (bool, Vec<String>) {
    let mut ignored_f = vec![];
    if let Ok(lines) = read_lines() {
        files.retain(|file| !ignore_file(file, lines.clone(), &mut ignored_f));
    }
    (ignored_f.len() > 0, ignored_f)
}

fn normalize_rule(l: String) -> String {
    let mut line = l;

    // define / as root
    let re = Regex::new("^(!)?/").unwrap();
    line = re.replace_all(&line, "$1^").to_string();

    // escape .
    let re = Regex::new(r"\.").unwrap();
    line = re.replace_all(&line, r"\.").to_string();

    // add . before *
    let re = Regex::new(r"\*").unwrap();
    line = re.replace_all(&line, r".*").to_string();

    // add optional .* at the end of / 
    let re = Regex::new(r"/$").unwrap();
    line = re.replace_all(&line, r"(/.*)?").to_string();
    line
}

pub fn ignore_file(path: &String, lines: Vec<String>, ignored_f: &mut Vec<String>) -> bool {
    let mut ignored = false;
    for line in lines {
        if line.starts_with("!") {
            if !ignored {
                continue;
            }
            let strip_line = line.strip_prefix("!").unwrap();
            let re = Regex::new(&strip_line).unwrap();
            if re.is_match(path) {
                ignored = false;
            }
        } else if !ignored {
            let re = Regex::new(&line).unwrap();
            if re.is_match(path) {
                ignored = true;
            }
        }
    }
    if ignored {
        ignored_f.push(path.to_string());
    }
    ignored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_files() {
        let lines_data = b"*.log\nexclude\n/logs/\n/build/target/\n**/*.swp\nsecret/\n";
        let cursor = Cursor::new(lines_data);
        let reader = BufReader::new(cursor);
        let mut lines = vec![];
        for line in reader.lines() {
            if let Ok(l) = line {
                lines.push(normalize_rule(l.clone()));
            }
        }
        let mut ignf = vec![];
        assert_eq!(ignore_file(&String::from("error.log"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("./error.log"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("dir/error.log"), lines.clone(), &mut ignf), true);

        assert_eq!(ignore_file(&String::from("exclude"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("dir/exclude"), lines.clone(), &mut ignf), true);

        assert_eq!(ignore_file(&String::from("logs/dir/file2"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("dir/logs/dir/file2"), lines.clone(), &mut ignf), false);

        assert_eq!(ignore_file(&String::from("build/target/file1"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("build/target/dir/file1"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("build"), lines.clone(), &mut ignf), false);
        assert_eq!(ignore_file(&String::from("build/target"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("dir/build/target"), lines.clone(), &mut ignf), false);

        assert_eq!(ignore_file(&String::from("dir/file.swp"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from(".swp"), lines.clone(), &mut ignf), false);

        assert_eq!(ignore_file(&String::from("secret"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("dir/secret"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("dir/secret/file"), lines.clone(), &mut ignf), true);
    }

    #[test]
    fn test_ignore_files_negation() {
        let lines_data = b"*\n!*.log\n!*log.*\n";
        let cursor = Cursor::new(lines_data);
        let reader = BufReader::new(cursor);
        let mut lines = vec![];
        for line in reader.lines() {
            if let Ok(l) = line {
                lines.push(normalize_rule(l.clone()));
            }
        }

        let mut ignf = vec![];
        assert_eq!(ignore_file(&String::from("file"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("dir/file"), lines.clone(), &mut ignf), true);
        assert_eq!(ignore_file(&String::from("file.log"), lines.clone(), &mut ignf), false);
        assert_eq!(ignore_file(&String::from("log.file"), lines.clone(), &mut ignf), false);
        assert_eq!(ignore_file(&String::from("dir/file.log"), lines.clone(), &mut ignf), false);
        assert_eq!(ignore_file(&String::from("dir/log.file"), lines.clone(), &mut ignf), false);
    }
}


