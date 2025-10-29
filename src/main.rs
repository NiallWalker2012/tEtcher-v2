/// # Main.rs
/// 
/// main.rs is the backbone for all other files' external functions
/// It includes the FFI C functions, and calls all rust external functions


use std::ffi::CString;
use std::io::{
    Result,
    Write,
    self
};
use std::os::raw::c_char;
use crossterm::{
    execute,
    cursor,
    terminal::{
        disable_raw_mode
    },
};
use std::time::Instant;
use std::process::exit;
use std::{
    thread,
    time::Duration
};

mod iso;
mod targ;
mod flash_confirm;
mod verify_confirm;

//Extern to initialize all C functions
unsafe extern "C" {
    fn flash(iso_path: *const c_char, dev_name: *const c_char);
    fn verify(iso_path: *const c_char, dev_name: *const c_char) -> bool;
}

fn main() -> Result<()> {
    let Some(iso_path) = iso::main()? else {
        eprintln!("\nFailed to get ISO file");
        return Ok(());
    };

    let dev_name = match targ::menu() {
        Ok(Some(dev)) => dev,
        Ok(None) => {
            eprintln!("NULL value found at dev_path: could not unwrap");
            return Ok(());
        }
        Err(why) => {
            eprintln!("Error getting device target: {why}");
            return Ok(());
        }
    };

    // Clear terminal
    println!("\x1B[H\x1B[2J");

    let mut stdout = std::io::stdout();

    execute!(stdout, cursor::Hide)?;

    let confirms_flash = flash_confirm::menu(&iso_path.display().to_string(), &dev_name);
    if !confirms_flash {
        disable_raw_mode()?;
        execute!(stdout, cursor::Show)?;
        exit(0);
    }

    let flash_time = Instant::now();

    //Convert iso_path and dev_name into a C string, to give the arguments for flash.c function
    let iso_c = CString::new(iso_path.to_string_lossy().into_owned()).unwrap();
    let dev_c = CString::new(dev_name.as_str()).unwrap();

    unsafe {
        //Call the flash function
        flash(iso_c.as_ptr(), dev_c.as_ptr());
    }

    let flash_time_taken = flash_time.elapsed();
    io::stdout().flush().unwrap();
    println!("\nFinished flashing in {:.2} seconds", flash_time_taken.as_secs_f64());

    thread::sleep(Duration::from_secs(3));

    let confirms_verify: bool = verify_confirm::menu(&iso_path.display().to_string(), &dev_name);
    if !confirms_verify {
        disable_raw_mode()?;
        execute!(stdout, cursor::Show)?;
        exit(0);
    }

    let is_verified: bool;

    unsafe {  
        is_verified = verify(iso_c.as_ptr(), dev_c.as_ptr());
    }
    if is_verified {
        println!("\nVerification success");
    } else {
        println!("\nVerification failed");
    }

    disable_raw_mode()?;
    execute!(stdout, cursor::Show)?;
    Ok(())
}
