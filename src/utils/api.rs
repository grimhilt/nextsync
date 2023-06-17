#[derive(Debug)]
pub struct ApiProps {
    pub host: String, // nextcloud.example.com
    pub username: String,
    pub root: String, // /dir/cloned
}

impl Clone for ApiProps {
    fn clone(&self) -> Self {
        ApiProps {
            host: self.host.to_string(),
            username: self.username.to_string(),
            root: self.root.to_string(),
        }
    }
}

pub fn get_relative_s(p: String, api_props: &ApiProps) -> String {
    let mut final_p = p.clone();
    final_p = final_p.strip_prefix("/remote.php/dav/files/").unwrap().to_string();
    final_p = final_p.strip_prefix(&api_props.username).unwrap().to_string();
    final_p = final_p.strip_prefix(&api_props.root).unwrap().to_string();
    final_p = final_p.strip_prefix("/").unwrap().to_string();
    final_p
}
