use std::path::PathBuf;
use indicatif::{ProgressBar, MultiProgress, ProgressStyle, HumanBytes};

use crate::utils::api::ApiProps;
use crate::services::api::ApiError;
use crate::services::download_files::DownloadFiles;
use crate::services::req_props::ObjProps;

const SIZE_TO_STREAM: u64 = 2 * 1024 * 1024;

pub struct Downloader {
    files: Vec<ObjProps>,
    should_log: bool,
    api_props: Option<ApiProps>,
    progress_bars: Vec<ProgressBar>,
    multi_progress: Option<MultiProgress>,
}

impl Downloader {
    pub fn new() -> Self {
        Downloader { 
            files: vec![],
            should_log: false,
            api_props: None,
            progress_bars: vec![],
            multi_progress: None,
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

    fn init_log(&mut self, nb_objs: u64, total_size: u64) {
        self.multi_progress = Some(MultiProgress::new());

        self.progress_bars.push(
            self.multi_progress
            .clone()
            .unwrap()
            .add(ProgressBar::new(nb_objs).with_message("Objects")));

        let msg = format!("0B/{}", HumanBytes(total_size).to_string());
        self.progress_bars.push(
            self.multi_progress
            .clone()
            .unwrap()
            .add(ProgressBar::new(total_size).with_message(msg)));

        self.progress_bars[0].set_style(
            ProgressStyle::with_template("{_:>10} [{bar:40}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "));

        self.progress_bars[1].set_style(
            ProgressStyle::with_template("[{elapsed_precise}] [{bar:40}] {msg}")
            .unwrap()
            .progress_chars("=> "));

        self.progress_bars[0].tick();
        self.progress_bars[1].tick();
    }

    fn update_bytes_bar(&self, size: u64) {
        let bytes_bar = &self.progress_bars[1];
        bytes_bar.inc(size);
        let msg = format!(
            "{}/{}",
            HumanBytes(bytes_bar.position()).to_string(),
            HumanBytes(bytes_bar.length().unwrap()).to_string());
        bytes_bar.set_message(msg);
    }

    pub fn download(&mut self, ref_p: PathBuf, callback: Option<&dyn Fn(ObjProps)>) {
        if self.should_log {
            let mut total_size = 0;
            let nb_objs = self.files.len();

            self.files
                .iter()
                .for_each(|f| 
                          if let Some(size) = f.contentlength { 
                              total_size += size 
                          }
                         );

            self.init_log(nb_objs.try_into().unwrap(), total_size); 
        }

        for file in self.files.clone() {
            let relative_s = &file.clone().relative_s.unwrap();
            let mut download = DownloadFiles::new();
            download.set_url(&relative_s, &self.api_props.clone().unwrap());

            let should_use_stream = {
                if let Some(size) = file.contentlength {
                    if size > SIZE_TO_STREAM {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            // download
            let res = {
                if should_use_stream {
                    download.save_stream(ref_p.clone(), Some(|a| self.update_bytes_bar(a)))
                } else {
                    download.save(ref_p.clone())
                }
            };

            // deal with error
            match res {
                Ok(()) => {
                    if let Some(fct) = callback {
                        fct(file.clone());
                    }
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

            // increment loading bars
            if self.should_log {
                self.progress_bars[0].inc(1); // increment object

                // increment bytes only if 
                // not incremented continuously by stream
                if !should_use_stream {
                    self.update_bytes_bar(file.contentlength.unwrap());
                }
            }
        }

        // finish all bars
        for bar in &self.progress_bars {
            bar.finish();
        }
    }
}
