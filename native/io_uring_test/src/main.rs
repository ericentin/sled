fn main() {
    match rio::Config::default().start() {
        Ok(_) => std::process::exit(0),
        Err(error) => {
            println!("{}", error);
            std::process::exit(1)
        }
    }
}
