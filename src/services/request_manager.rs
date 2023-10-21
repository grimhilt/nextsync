use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::services::login::Login;
use crate::commands::config;
use crate::commands::clone::get_url_props;
use crate::services::api_call::ApiCall;

lazy_static! {
    static ref REQUEST_MANAGER: Mutex<Option<RequestManager>> = Mutex::new(None);
}

pub fn get_request_manager() -> &'static Mutex<Option<RequestManager>> {
    if REQUEST_MANAGER.lock().unwrap().is_none() {
        *REQUEST_MANAGER.lock().unwrap() = Some(RequestManager::new());
    }
    &REQUEST_MANAGER
}

pub struct RequestManager {
    token: Option<String>,
    host: Option<String>,
}

impl RequestManager {
    pub fn new() -> Self {
        RequestManager {
            token: None,
            host: None,
        }
    }

    pub fn set_host(&mut self, host: String) {
        self.host = Some(host);
    }

    pub fn get_host(&mut self) -> String
    {
        if self.host.is_none()
        {
            let remote = match config::get("remote") {
                Some(r) => r,
                None => {
                    // todo ask user instead
                    eprintln!("fatal: unable to find a remote");
                    std::process::exit(1);
                }
            };
            let (host, _, _) = get_url_props(&remote);
            self.host = Some(host.clone());
            // todo ask user
        }
        self.host.clone().unwrap()
    }

    pub fn get_token(&mut self) -> String {
        if self.token.is_none() {
            // todo check in config
            let get_token = Login::new()
                .ask_auth()
                .set_host(Some(self.get_host()))
                .send_login();
            // todo deal with error cases
            self.token = Some(get_token.unwrap());
        }
        self.token.clone().unwrap()
    }

    pub fn create_request()
    {

    }
}
