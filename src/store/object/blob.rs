use std::io;
use std::path::PathBuf;
use std::time::SystemTime;
use std::fs;
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use crate::utils::path;
use crate::store::head;
use crate::store::object::{update_dates, add_node, create_obj, rm_node};

pub struct Blob {
    path: PathBuf,
    hash: String,
    obj_p: PathBuf,
}

impl Blob {
    pub fn new(path: PathBuf) -> Blob {
        let mut hasher = Sha1::new();
        hasher.input_str(path.to_str().unwrap());
        let hash = hasher.result_str();

        let (dir, res) = hash.split_at(2);
        
        let mut obj_p = path::objects();
        obj_p.push(dir);
        obj_p.push(res);

        Blob {
            path,
            hash,
            obj_p,
        } 
    }

    fn get_line_filename(&mut self) -> (String, String) {
        let file_name = self.path.file_name().unwrap().to_str().unwrap().to_owned();
        let mut line = String::from("blob");
        line.push_str(" ");
        line.push_str(&self.hash);
        line.push_str(" ");
        line.push_str(&file_name);
        (line, file_name)
    }

    fn get_file_hash(&self) -> String {
        let bytes = std::fs::read(self.path.clone()).unwrap();
        let hash = md5::compute(&bytes);
        format!("{:x}", hash)
    }

    pub fn create(&mut self, ts_remote: &str, up_parent: bool) -> io::Result<()> {
        let (line, file_name) = self.get_line_filename();

        // add blob reference to parent
        if self.path.iter().count() == 1 {
            head::add_line(line)?;
        } else {
            add_node(self.path.parent().unwrap(), &line)?;
        }

        // create blob object
        let metadata = fs::metadata(self.path.clone())?;

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
        create_obj(self.hash.clone(), &content)?;

        // update date for all parent
        if up_parent {
            update_dates(self.path.clone(), ts_remote)?;
        }
        Ok(())
    }

    pub fn rm(&mut self) -> io::Result<()> {
        let (line, _) = self.get_line_filename();

        // remove blob reference to parent
        if self.path.iter().count() == 1 {
            head::rm_line(&line)?;
        } else {
            rm_node(self.path.parent().unwrap(), &line)?;
        }

        // remove blob object
        fs::remove_file(self.obj_p.clone())?;

        Ok(())
    }
}
