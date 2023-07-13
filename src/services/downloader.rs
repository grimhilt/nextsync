use std::path::PathBuf;
use crate::utils::api::ApiProps;
use crate::services::api::ApiError;
use crate::services::download_files::DownloadFiles;
use crate::services::req_props::ObjProps;

const SIZE_TO_STREAM: u64 = 2 * 1024 * 1024;

pub struct Downloader {
    files: Vec<ObjProps>,
    should_log: bool,
    api_props: Option<ApiProps>,
}

impl Downloader {
    pub fn new() -> Self {
        Downloader { 
            files: vec![],
            should_log: false,
            api_props: None,
        }
    } 

    pub fn should_log(&mut self) -> &mut Downloader {
        self.should_log = true;
        self
    }

    pub fn set_api_props(&mut self, api_props: ApiProps) -> &mut Downloader {
        self.api_props = Some(api_props);
        self
    }

    pub fn set_files(&mut self, files: Vec<ObjProps>) -> &mut Downloader {
        self.files = files;
        self
    }

    pub fn add_file(&mut self, file: ObjProps) -> &mut Downloader {
        self.files.push(file);
        self
    }

    pub fn download(&mut self, ref_p: PathBuf, callback: Option<&dyn Fn(ObjProps)>) {
        for file in self.files.clone() {
            let relative_s = &file.clone().relative_s.unwrap();
            let mut download = DownloadFiles::new();
            download.set_url(&relative_s, &self.api_props.clone().unwrap());

            let res = {
                if let Some(size) = file.contentlength {
                    if size > SIZE_TO_STREAM {
                        download.save_stream(ref_p.clone())
                    } else {
                        download.save(ref_p.clone())
                    }
                } else {
                    download.save(ref_p.clone())
                }
            };

            
            match res {
                Ok(()) => {
                    if let Some(fct) = callback {
                        fct(file);
                    }
                    //let relative_p = PathBuf::from(&relative_s);
                    //let lastmodified = obj.clone().lastmodified.unwrap().timestamp_millis();
                    //if let Err(err) = blob::add(relative_p, &lastmodified.to_string()) {
                    //    eprintln!("err: saving ref of {} ({})", relative_s.clone(), err);
                    //}
                },
                Err(ApiError::Unexpected(_)) => {
                    eprintln!("err: writing {}", relative_s);
                },
                Err(ApiError::IncorrectRequest(err)) => {
                    eprintln!("fatal: {}", err.status());
                    std::process::exit(1);
                },
                Err(ApiError::EmptyError(_)) => eprintln!("Failed to get body"),
                Err(ApiError::RequestError(err)) => {
                    eprintln!("fatal: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}
