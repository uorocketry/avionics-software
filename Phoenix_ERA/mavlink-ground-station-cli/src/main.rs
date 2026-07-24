use std::process::Command;
use std::{ time::Duration};
use std::io::{Write, stdout};

use mavlink;
use tokio::{
    io::{AsyncRead, AsyncReadExt, Stdin, stdin}, sync::Mutex, task, time
};
use std::sync::LazyLock;


static USER_INPUT: LazyLock<Mutex<String>> = LazyLock::new(||{Mutex::new(String::new())});

#[tokio::main]
async fn main() {


    let join = task::spawn(menu());
    let join_2 = task::spawn(update_stdin());
    loop {}
}

async fn menu() {
    {

    println!("ENTER DESIRED SELECTION: ");
    println!("\t1: Show current rx buffer");
    println!("\t2: Transmit command");
    loop {
        time::sleep(Duration::from_secs(1)).await;
        
        if let Ok(val) = USER_INPUT.lock().await.as_str().trim().parse::<u32>() {
            println!("=={}==",val);
        match val {
            1 => {
                println!("INVALID");
                time::sleep(Duration::from_secs(2));
                // clear_terminal_screen();
            }
            _ => ()
        
        }
    }
    }
    }
}

async fn rx_buffer_menu() {

}

async fn update_stdin() {
    let mut stdin = stdin();
    loop {
        let mut buff: [u8; 256] = [0; 256];
        println!("Enter a value: ");
        let bytes_read = stdin.read(&mut buff).await.unwrap();
        if let Ok(string) = str::from_utf8(&buff){ 
            *USER_INPUT.lock().await = string.to_string();
        }
    }
}
pub fn clear_terminal_screen() {
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "cls"]).spawn()
    } else {
        // "clear" or "tput reset"
        Command::new("tput").arg("reset").spawn()
    };

    // Alternative solution:
    if result.is_err() {
        print!("{esc}c", esc = 27 as char);
    }
}