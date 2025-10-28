#![allow(unused_doc_comments)]

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Stylize},
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::fs;
use std::io::{stdout, Write};

/// To run this program, go to the README.md and follow the steps
/// 
/// This program will list all directories and files, letting you move around your computer's files like a file explorer
/// When a file is chosen, it will ask if '/folder/file' is the correct path, and then proceed to targ.rs
/// 
/// Navigate using the arrow-keys to move up and down, and enter key to select
pub fn main() -> std::io::Result<Option<std::path::PathBuf>> {
    let mut selected = 0;
    let mut current_dir = std::env::current_dir()?; // Track current directory

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, cursor::Hide)?;

    loop {
        // Read all entries in the current directory
        let mut menu_items: Vec<String> = if let Ok(entries) = fs::read_dir(&current_dir) {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect()
        } else {
            vec!["Error reading directory".to_string()]
        };

        // Added "[Exit]" option for easy exit access
        menu_items.insert(0, "[Exit]".to_string());

        // Add a "Back" option if not at root
        if current_dir.parent().is_some() {
            menu_items.insert(1, "[Back]".to_string());
        }

        // Adjust selection if needed
        if selected >= menu_items.len() {
            selected = menu_items.len().saturating_sub(1);
        }

        // Draw UI
        execute!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::FromCursorDown)
        )?;
        println!("{}", "Please navigate to the file you wish to flash".with(Color::Blue));

        for (i, item) in menu_items.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, (i + 1) as u16))?;
            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;

            // Determine styling
            let display_item = if item == "[Back]" {
                item.clone().with(Color::Green).bold().to_string()
            } else if item == "[Exit]" {
                item.clone().with(Color::Red).bold().to_string()
            } else if current_dir.join(item).is_dir() {
                item.clone().with(Color::Blue).bold().to_string()
            } else {
                item.clone()
            };

            // Highlight selected option
            if i == selected {
                print!("  {}", display_item.on_white().black());
            } else {
                print!("  {}", display_item);
            }
        }

        stdout.flush()?;

        // Handle user input
        if let Event::Key(event) = event::read()? {
            match event.code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected < menu_items.len().saturating_sub(1) {
                        selected += 1;
                    }
                }
                KeyCode::Enter => {
                    let selected_item = &menu_items[selected];
                    if selected_item == "[Exit]" {
                        execute!(stdout, cursor::Show)?;
                        disable_raw_mode()?;
                        return Ok(None)
                    } else if selected_item == "[Back]" {
                        if let Some(parent) = current_dir.parent() {
                            current_dir = parent.to_path_buf();
                            selected = 0;
                        }
                        continue;
                    }

                    let path = current_dir.join(selected_item);
                    if path.is_dir() {
                        current_dir = path;
                        selected = 0;
                        continue;
                    }

                    // File selected: confirmation
                    let confirm_options = ["Yes", "No"];
                    let mut confselected = 0;

                    loop {
                        execute!(
                            stdout,
                            cursor::MoveTo(0, 0),
                            terminal::Clear(ClearType::FromCursorDown)
                        )?;
                        println!("Is '{}' the file you wish to flash?", path.display());

                        for (i, item) in confirm_options.iter().enumerate() {
                            execute!(stdout, cursor::MoveTo(0, (i + 2) as u16))?;
                            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
                            if i == confselected {
                                print!("  {}", item.on_white().black());
                            } else {
                                print!("  {}", item);
                            }
                        }

                        stdout.flush()?;

                        if let Event::Key(ev) = event::read()? {
                            match ev.code {
                                KeyCode::Up => {
                                    if confselected > 0 {
                                        confselected -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    if confselected < confirm_options.len() - 1 {
                                        confselected += 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    if confirm_options[confselected] == "Yes" {
                                        // File confirmed, exit program
                                        execute!(stdout, cursor::Show)?;
                                        disable_raw_mode()?;
                                        return Ok(Some(path));
                                    }
                                    break;
                                }
                                KeyCode::Esc => break,
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
