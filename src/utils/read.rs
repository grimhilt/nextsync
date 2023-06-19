use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn read_folder(path: PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
 
    entries.sort();
    Ok(entries)
}


pub fn rm_line(path: PathBuf, line_to_del: &str) -> io::Result<()> {
    let file = File::open(path.clone())?;
    let reader = BufReader::new(&file);
    let mut temp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{}_temp", path.display()))?;

    for line in reader.lines() {
        let l = line?;
        if l.trim() != line_to_del.trim() {
            writeln!(temp_file, "{}", l)?;
        }
    }

    drop(file);
    drop(temp_file);

    fs::remove_file(path.clone())?;
    fs::rename(format!("{}_temp", path.display()), path)?;
    Ok(())
}
