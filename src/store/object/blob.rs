use std::io::{self, Read};
use std::fs::{self, File};
use std::io::Write;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::SystemTime;
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use crate::commands::status::{LocalObj, State};
use crate::utils::path::path_buf_to_string;
use crate::utils::{path, read};
use crate::store::head;
use crate::store::object::{update_dates, add_node, rm_node};

const HASH_EMPTY: &str = "d41d8cd98f00b204e9800998ecf8427e";

pub struct Blob {
    r_path: PathBuf, // relative path
    a_path: PathBuf, // absolute path
    hash: String, // hash of relative path
    file_hash: Option<String>,
    obj_p: PathBuf, // path of the object file
    data: Vec<String>, // content of the blob
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
            file_hash: None,
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

    fn get_file_hash(&mut self) -> String {
        if self.file_hash.is_none() {
            let bytes = std::fs::read(self.a_path.clone()).unwrap();
            let hash = md5::compute(&bytes);
            self.file_hash = Some(format!("{:x}", hash))
        }
        self.file_hash.clone().unwrap()
    }

    fn create_blob_ref(&mut self, file_name: String, ts_remote: &str) -> io::Result<()> {
        let metadata = fs::metadata(self.a_path.clone())?;
        let secs = metadata
            .modified()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut content = file_name.clone();
        content.push_str(" ");
        content.push_str(ts_remote);
        content.push_str(" ");
        content.push_str(&metadata.len().to_string());
        content.push_str(" ");
        content.push_str(&secs.to_string());
        content.push_str(" ");
        content.push_str(&self.get_file_hash());

        let binding = self.obj_p.clone();
        let child = binding.file_name();
        self.obj_p.pop();
        if !self.obj_p.clone().exists() {
           fs::create_dir_all(self.obj_p.clone())?; 
        }
        self.obj_p.push(child.unwrap().to_str().unwrap());

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(self.obj_p.clone())?;

        writeln!(file, "{}", &content)?;

        Ok(())
    }

    fn get_file_ref(&mut self) -> PathBuf {
        let mut refs_p = path::refs();
        let file_hash = self.get_file_hash().clone();
        let (dir, res) = file_hash.split_at(2);

        refs_p.push(dir);
        if !refs_p.exists() {
           fs::create_dir_all(refs_p.clone()); 
        }
        refs_p.push(res);
        refs_p
    }

    // create a file in .nextsync/refs with the hash of this blob that
    // redirect to the relative path
    fn create_hash_ref(&mut self) -> io::Result<()> {
        // todo check if the file has been modified for moved and copy 
        let refs_p = self.get_file_ref();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(refs_p)?;

        // todo deal with duplicate content
        writeln!(file, "{}", self.r_path.clone().to_str().unwrap())?;
        Ok(())
    }

    pub fn get_all_identical_blobs(&mut self) -> Vec<String> {
        // an empty file is a new file not the copy of another empty file
        if self.get_file_hash() == HASH_EMPTY {
            return vec![];
        }

        let refs_p = self.get_file_ref();
        let mut blobs: Vec<String> = vec![];
        if let Ok(lines) = read::read_lines(refs_p) {
            for line in lines {
                if let Ok(l) = line {
                    blobs.push(l.clone());
                }
            } 
        }
        blobs
    }

    pub fn create(&mut self, ts_remote: &str, up_parent: bool) -> io::Result<()> {
        let (line, file_name) = self.get_line_filename();

        // add blob reference to parent
        if self.r_path.iter().count() == 1 {
            head::add_line(line)?;
        } else {
            add_node(self.r_path.parent().unwrap(), &line)?;
        }

        if let Err(err) = self.create_blob_ref(file_name.clone(), ts_remote.clone()) {
            eprintln!("err: saving blob ref of {}: {}", self.r_path.clone().display(), err);
        }

        if let Err(err) = self.create_hash_ref() {
            eprintln!("err: saving hash ref of {}: {}", self.r_path.clone().display(), err);
        }

        // update date for all parent
        if up_parent {
            if let Err(err) = update_dates(self.r_path.clone(), ts_remote) {
                eprintln!("err: updating parent date of {}: {}", self.r_path.clone().display(), err);
            }
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

    pub fn update(&mut self, ts_remote: &str) -> io::Result<()> {

        // remove old hash ref
        let mut refs_p = path::refs();
        let binding = self.saved_hash();
        let (dir, res) = binding.split_at(2);
        refs_p.push(dir);
        refs_p.push(res);
        if let Err(err) = fs::remove_file(refs_p) {
            eprintln!("err: removing hash ref of {}: {}", self.r_path.clone().display(), err);
        }

        // creating new hash ref
        if let Err(err) = self.create_hash_ref() {
            eprintln!("err: saving hash ref of {}: {}", self.r_path.clone().display(), err);
        }

        // updating content of blob's ref
        let metadata = fs::metadata(self.a_path.clone())?;
        let secs = metadata
            .modified()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut content = self.saved_filename();
        content.push_str(" ");
        content.push_str(ts_remote);
        content.push_str(" ");
        content.push_str(&metadata.len().to_string());
        content.push_str(" ");
        content.push_str(&secs.to_string());
        content.push_str(" ");
        content.push_str(&self.get_file_hash());

        let mut file = OpenOptions::new()
            .write(true)
            .open(self.obj_p.clone())?;

        writeln!(file, "{}", &content)?;
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

                if let Some(last) = self.data.last_mut() {
                    if last.ends_with("\n") {
                        last.pop();
                    }
                }
            }
        }
    }

    fn saved_filename(&mut self) -> String {
        self.read_data();
        if self.data.len() >= 1 { 
            self.data[0].clone()
        } else {
            String::new()
        }
    }

    pub fn saved_remote_ts(&mut self) -> String {
        self.read_data();
        if self.data.len() >= 2 { 
            self.data[1].clone()
        } else {
            String::new()
        }
    }

    fn saved_local_size(&mut self) -> String {
        self.read_data();
        if self.data.len() >= 3 { 
            self.data[2].clone()
        } else {
            String::new()
        }
    }

    fn saved_local_ts(&mut self) -> u64 {
        self.read_data();
        if self.data.len() >= 4 { 
            self.data[3].parse::<u64>().unwrap()
        } else {
            0
        }
    }

    fn saved_hash(&mut self) -> String {
        self.read_data();
        if self.data.len() >= 5 { 
            self.data[4].clone()
        } else {
            String::new()
        }
    }

    fn has_same_size(&mut self) -> bool {
        let metadata = match fs::metadata(self.a_path.clone()) {
            Ok(m) => m,
            Err(_) => return true,
        };

        if self.saved_local_size() == String::new() { return true; }
        metadata.len().to_string() == self.saved_local_size()
    }

    fn is_newer(&mut self) -> bool {
        let metadata = match fs::metadata(self.a_path.clone()) {
            Ok(m) => m,
            Err(_) => return true,
        };

        let secs = metadata
            .modified()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if self.saved_local_ts() == 0 { return true; }
        secs > self.saved_local_ts()
    }

    fn has_same_hash(&mut self) -> bool {
        if self.saved_hash() == String::new() { return false; }
        let file_hash = self.get_file_hash().clone();
        self.saved_hash() == file_hash
    }

    pub fn has_change(&mut self) -> bool {
        !self.has_same_size() || (self.is_newer() && !self.has_same_hash())
    }

    pub fn get_local_obj(&mut self) -> LocalObj {
        let mut path_from = None;
        let state = {
            let has_obj_ref = self.obj_p.clone().exists();
            let blob_exists = self.a_path.clone().exists();

            if has_obj_ref && !blob_exists {
                State::Deleted
            } else if !has_obj_ref && blob_exists {
                let identical_blobs = self.get_all_identical_blobs();
                if identical_blobs.len() != 0 {
                    let identical_blob = Blob::new(identical_blobs[0].clone().into())
                        .get_local_obj();
                    if identical_blob.state == State::Deleted {
                        path_from = Some(identical_blob.path);
                        State::Moved
                    } else if identical_blob.state == State::Default {
                        path_from = Some(identical_blob.path);
                        State::Copied
                    } else {
                        State::New
                    }
                } else {
                    State::New
                }
            } else if !has_obj_ref && !blob_exists {
                State::Default
            } else if self.has_change() {
                State::Modified
            } else {
                State::Default
            }
        };

        LocalObj { 
            otype: String::from("blob"),
            name: path_buf_to_string(self.r_path.clone()),
            path: self.r_path.clone(),
            path_from,
            state 
        }
    }
}

