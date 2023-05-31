use std::fs::DirBuilder;

pub fn init() {
    let builder = DirBuilder::new();
    match builder.create("./test/dir") {
        Ok(()) => println!("Directory successfuly created"),
        Err(_) => println!("Error: cannot create directory"),
    }
}
