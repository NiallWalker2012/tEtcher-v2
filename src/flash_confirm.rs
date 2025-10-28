use crossterm::{
    cursor,
    event::{
        self,
        Event,
        KeyCode
    },
    execute,
    style::{
        Stylize
    },
    terminal::{
        self,
        ClearType,
        disable_raw_mode,
        enable_raw_mode
    },
};
use std::io::{
    stdout,
    Write
};


pub fn menu(iso: &str, dev: &str) -> bool {
    enable_raw_mode().unwrap();
    let mut stdout = stdout();

    let warn = vec!["Yes", "No"];
    let mut selected = 0;

    loop {
        execute!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::FromCursorDown)).unwrap();
        println!(
            "Do you wish to flash {} to {}? THIS WILL OVERWRITE *ALL* DISK CONTENTS",
            iso, dev
        );

        for (i, item) in warn.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, (i + 1) as u16)).unwrap();
            execute!(stdout, terminal::Clear(ClearType::CurrentLine)).unwrap();

            if i == selected {
                print!("{}", item.on_white().black());
            } else {
                print!("{}", item);
            }
        }

        stdout.flush().unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Up => if selected > 0 { selected -= 1 },
                KeyCode::Down => if selected < warn.len() - 1 { selected += 1 },
                KeyCode::Enter => {
                    disable_raw_mode().unwrap();
                    return selected == 0;
                }
                KeyCode::Esc => {
                    disable_raw_mode().unwrap();
                    return false;
                }
                _ => {}
            }
        }
    }
}
