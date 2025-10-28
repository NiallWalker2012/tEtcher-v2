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

mod iso;
mod targ;
mod flash_confirm;

unsafe extern "C" {
    fn flash(iso_path: *const c_char, dev_name: *const c_char);
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

    let confirms = flash_confirm::menu(&iso_path.display().to_string(), &dev_name);
    if !confirms {
        exit(0);
    }

    let time = Instant::now();

    unsafe {
        let iso_c = CString::new(iso_path.to_string_lossy().into_owned()).unwrap();
        let dev_c = CString::new(dev_name).unwrap();
        flash(iso_c.as_ptr(), dev_c.as_ptr());
    }

    let time_taken = time.elapsed();
    io::stdout().flush().unwrap();
    println!("\nFinished flashing in {:.2} seconds", time_taken.as_secs_f64());


    disable_raw_mode()?;
    execute!(stdout, cursor::Show)?;
    Ok(())
}
