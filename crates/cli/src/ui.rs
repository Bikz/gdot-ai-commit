pub fn info(message: &str) {
    println!("{message}");
}

pub fn warn(message: &str) {
    eprintln!("warning: {message}");
}

pub fn error(message: &str) {
    eprintln!("error: {message}");
}

pub fn success(message: &str) {
    println!("{message}");
}

pub fn divider() {
    println!("----------------------------------------------------------------");
}

pub fn preview_message(message: &str) {
    divider();
    println!("{message}");
    divider();
}
