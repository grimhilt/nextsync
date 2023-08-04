use std::io::{self, Read};
use std::fs::File;
use std::io::Write;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::SystemTime;
use std::fs;
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use crate::utils::path;
use crate::store::head;
use crate::store::object::{update_dates, add_node, create_obj, rm_node};

pub struct Blob {
    r_path: PathBuf,
    a_path: PathBuf,
    hash: String,
    obj_p: PathBuf,
    data: Vec<String>,
}

impl Blob {
    pub fn new(r_path: PathBuf) -> Blob {
        let mut hasher = Sha1::new();
        hasher.input_str(r_path.to_str().unwrap());
        let hash = hasher.result_str();

        let (dir, res) = hash.split_at(2);
        
        let mut obj_p = path::objects();
        obj_p.push(dir);
        obj_p.push(res);

        let root = path::repo_root();
        let a_path = root.join(r_path.clone());

        Blob {
            r_path,
            a_path,
            hash,
            obj_p,
            data: vec![],
        } 
    }

    fn get_line_filename(&mut self) -> (String, String) {
        let file_name = self.r_path.file_name().unwrap().to_str().unwrap().to_owned();
        let mut line = String::from("blob");
        line.push_str(" ");
        line.push_str(&self.hash);
        line.push_str(" ");
        line.push_str(&file_name);
        (line, file_name)
    }

    fn get_file_hash(&self) -> String {
        let bytes = std::fs::read(self.a_path.clone()).unwrap();
        let hash = md5::compute(&bytes);
        format!("{:x}", hash)
    }

    pub fn create(&mut self, ts_remote: &str, up_parent: bool) -> io::Result<()> {
        let (line, file_name) = self.get_line_filename();

        // add blob reference to parent
        if self.r_path.iter().count() == 1 {
            head::add_line(line)?;
        } else {
            add_node(self.r_path.parent().unwrap(), &line)?;
        }

        // create blob object
        let metadata = fs::metadata(self.a_path.clone())?;

        let mut content = file_name.clone();
        content.push_str(" ");
        content.push_str(ts_remote);
        content.push_str(" ");
        content.push_str(&metadata.len().to_string());
        content.push_str(" ");

        let secs = metadata
            .modified()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        content.push_str(&secs.to_string());
        content.push_str(" ");
        content.push_str(&self.get_file_hash());
        content.push_str(" ");

        // create ref of object
        let binding = self.obj_p.clone();
        let child = binding.file_name();
        self.obj_p.pop();
        self.obj_p.push(child.unwrap().to_str().unwrap());
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(self.obj_p.clone())?;
        writeln!(file, "{}", &content)?;

        // update date for all parent
        if up_parent {
            update_dates(self.r_path.clone(), ts_remote)?;
        }
        Ok(())
    }

    pub fn rm(&mut self) -> io::Result<()> {
        let (line, _) = self.get_line_filename();

        // remove blob reference to parent
        if self.r_path.iter().count() == 1 {
            head::rm_line(&line)?;
        } else {
            rm_node(self.r_path.parent().unwrap(), &line)?;
        }

        // remove blob object
        fs::remove_file(self.obj_p.clone())?;

        Ok(())
    }

    pub fn read_data(&mut self) {
        if self.data.len() == 0 {
            if let Ok(mut file) = File::open(self.obj_p.clone()) {
                let mut buffer = String::new();
                let _ = file.read_to_string(&mut buffer);
                let data = buffer.rsplit(' ').collect::<Vec<_>>();
                for e in data {
                    self.data.push(String::from(e));
                }
                self.data.reverse();
            }
        }
    }

    fn has_same_size(&mut self) -> bool {
        let metadata = match fs::metadata(self.a_path.clone()) {
            Ok(m) => m,
            Err(_) => return true,
        };

        self.read_data();
        if self.data.len() < 3 { return true; }
        metadata.len().to_string() == self.data[2]
    }

    fn is_newer(&mut self) -> bool {
        let metadata = match fs::metadata(self.a_path.clone()) {
            Ok(m) => m,
            Err(_) => return true,
        };

        self.read_data();
        let secs = metadata
            .modified()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if self.data.len() < 4 { return true; }
        secs > self.data[3].parse::<u64>().unwrap()
    }

    fn has_same_hash(&mut self) -> bool {
        self.read_data();
        if self.data.len() < 5 { return false; }
        self.data[4] == self.get_file_hash()
    }

    pub fn has_change(&mut self) -> bool {
        !self.has_same_size() || (self.is_newer() && !self.has_same_hash())
    }
}

