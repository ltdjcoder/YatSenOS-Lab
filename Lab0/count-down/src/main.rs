use std::time::Duration;
use std::thread::sleep;
use std::fs::File;
use std::io::Read;
use std::fs::metadata;


fn count_down(mut seconds: u64){
    while seconds > 0 {
        println!("{} seconds remaining", seconds);
        sleep(Duration::from_secs(1));
        seconds -= 1;
    }
    println!("Countdown finished!");
}

fn read_and_print(file_path: &str) {
    let mut file: File = File::open(file_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read file");
    println!("{}", contents);
}

fn file_size(file_path: &str) -> Result<u64, &str> {
    match metadata(file_path) {
        Ok(meta) => Ok(meta.len()),
        Err(_) => Err("File not found!"),
    }
}

fn main() {
    count_down(3);
    let result = file_size("test1.txt");
    match result {
        Ok(size) => println!("File size: {} bytes", size),
        Err(e) => println!("{}", e),
    }
    read_and_print("test1.txt");
}
