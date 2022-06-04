// Project: less_fp - Simple less with fixation points.
// Author: João Nuno Carvalho
// Data: 2022.06.04
// Description: This simple program reads a text file, paginate it and shows
//              it with fixation points in bold. In principal they are
//              supposed to allow you to read faster.
//              Use Esc to quit.
//              Use 'q' to prev_page.
//              Use 'a' to next_page.
//              Use mouse or keyboard for terminal resize.
//              I tested it under Linux.
//
// License: MIT Open Source license.
//
// Have fun!
//
// TODO: 
//    -Implement simple Search keys / + text_to_search + enter, with next and previous key bindings,
//    the found words will be negative highlighted. The pages will automatically jump
//    to the next or the previous page to go to the nearest word.
//
use clap::Parser;
use std::path::PathBuf;
use std::fs;

mod string_utils;
use string_utils::{StringUtils /* , StringUtilsVecCharsV2*/ };

use std::io::{stdout /*, Stdout, Write */};

use crossterm::event::poll;
use crossterm::style::Color;
use crossterm::{
    cursor::{position, MoveTo},
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
    Result,
};
use crossterm::ExecutableCommand;
use crossterm::style::{Color::Rgb, Color::*, Colors, Print, SetColors, Stylize};

use std::time::Duration;

use crate::string_utils::StringUtilsVecCharsV2;

const HELP: &str = r#"
 - Keyboard, mouse and terminal resize events enabled
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

const COLOR_REAL_BLACK: Color = Rgb {r: 0, g: 0, b: 0};

/// This simple program reads a text file, paginate it and shows it with
/// fixation points in bold. In principal they are supposed to allow you
/// to read faster.
/// Use Esc to quit.
/// Use 'q' to prev_page.
/// Use 'a' to next_page.
/// Use mouse or keyboard for terminal resize.
///
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Text file
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    file: Option<PathBuf>,

    // TODO:
    // Read arguments from command line : filename, color, page, Search pattern and next and previous.
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Checking the path of the text file.
    match args.file.as_deref() {
        Some(file) => {
                if !file.exists() {
                    println!("Error: The text filename '{}' doesn't exist!", file.to_string_lossy());
                    return Ok(());
                    // TODO: Add proper exit code with error.
                }
                println!("Text filename: {}", file.to_string_lossy());
                match fs::read_to_string(file) {
                    // And converts the String to a String Vector. 
                    Ok(text) => {
                            let mut text_vec = text.get_vec_chars();
                            text_vec.replace_str_all("\r\n", "\n");
                            start_text_mode(&text_vec)?;
                        },
                    // TODO: Add proper exit code with error.
                    Err(err_str) => println!("Error: While reading file ... {}", err_str),
                }
                Ok(())
            },
        None => {
                println!("Please enter a file, see option --help .");
                Ok(())
            },
    }
}

fn start_text_mode(text_vec: &Vec<char>) -> Result<()> {
    println!("Quick reading with fixation points.");

    println!("{}", HELP);

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let mut pages_vec = PageVec::paginate(text_vec, terminal::size());
    let (_page_num, page) = pages_vec.get_curr_page();
    let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
    print_fp(&text_vec_page /* , Green */ );

    if let Err(e) = print_events(text_vec, & mut pages_vec) {
        println!("Error: {:?}\r", e);
    }

    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()

}

fn print_events(text_vec: &Vec<char>, pages_vec: & mut PageVec) -> Result<()> {
    loop {
        // Blocking read
        let event = read()?;
        // println!("Event: {:?}\r", event);

        if event == Event::Key(KeyCode::Char('c').into()) {
            println!("Cursor position: {:?}\r", position());
        }

        if let Event::Resize(_, _) = event {
            let (original_size, new_size) = flush_resize_events(event);
            println!("Resize from: {:?}, to: {:?}", original_size, new_size);
            stdout().execute(terminal::Clear(terminal::ClearType::All)).unwrap();

            // Get the old text char position.
            let (_page_num, page) = pages_vec.get_curr_page();
            let cur_start_page_char_pos = page.global_start_char_pos;

            // Do the new pagination.
            *pages_vec = PageVec::paginate(text_vec, terminal::size());
            
            // Find the new page inside the new pagination.
            let target_page = pages_vec.find_char_pos_in_pages(cur_start_page_char_pos);

            // Set the found page as the current page.
            pages_vec.set_curr_page_num(target_page);

            let (_page_num, page) = pages_vec.get_curr_page();        
            let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
            print_fp(&text_vec_page);
        }

        if event == Event::Key(KeyCode::Esc.into()) {
            execute!(stdout(), terminal::Clear(terminal::ClearType::All), MoveTo(0, 0)).unwrap();
            break;
        }

        if event == Event::Key(KeyCode::Char('q').into()) && pages_vec.prev_page(){
            let (_page_num, page) = pages_vec.get_curr_page();        
            let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
            print_fp(&text_vec_page);
        }

        if event == Event::Key(KeyCode::Char('a').into()) && pages_vec.next_page() {
            let (_page_num, page) = pages_vec.get_curr_page();        
            let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
            print_fp(&text_vec_page);
        }
    }

    Ok(())
}

// Resize events can occur in batches.
// With a simple loop they can be flushed.
// This function will keep the first and last resize event.
fn flush_resize_events(event: Event) -> ((u16, u16), (u16, u16)) {
    if let Event::Resize(x, y) = event {
        let mut last_resize = (x, y);
        while let Ok(true) = poll(Duration::from_millis(50)) {
            if let Ok(Event::Resize(x, y)) = read() {
                last_resize = (x, y);
            }
        }

        return ((x, y), last_resize);
    }
    ((0, 0), (0, 0))
}

struct Page {
    global_start_char_pos: usize,
    global_stop_char_pos: usize,
}

struct PageVec {
    curr_page: usize,
    pages_vec: Vec<Page>,
}

impl PageVec {
    fn paginate(text_vec: &Vec<char>, new_size: Result<(u16, u16)>) -> Self {
        let curr_page: usize = 0;
        let mut page = Some(Page {global_start_char_pos: 0, global_stop_char_pos: 0});
        let mut pages_vec: Vec<Page> = Vec::new();
 
        let mut cur_row = 0_u16;
        let mut cur_column = 0_u16;

        let (max_colum, max_row) = new_size.unwrap();
        let (max_colum, max_row) = (max_colum - 1, max_row - 1);

        // Paginates - Divide the Vec<chars> into the needed pages for the current size.
        for (i, c) in text_vec.iter().enumerate() {
            if *c != '\n' {
                if cur_column < max_colum {
                    cur_column += 1; 
                } else {
                    if max_row == cur_row {
                        // The text spills to the next page.
                        // Creates the new page.
                        if let Some(mut page_tmp) = page {
                            page_tmp.global_stop_char_pos = i - 1;
                            pages_vec.push(page_tmp);
                            page = Some(Page {global_start_char_pos: i, global_stop_char_pos : i});
                        }
                        cur_row = 0;
                    } else {
                        // Continue on the same page.
                        cur_row += 1;
                    }
                    cur_column = 0;
                }
            } else {
                // New line character.
                if max_row == cur_row {
                    // The text spills to the next page.
                    // Creates the new page.
                    if let Some(mut page_tmp) = page {
                        page_tmp.global_stop_char_pos = i - 1;
                        pages_vec.push(page_tmp);
                        if i < text_vec.len() - 1 {
                            // So that /n isn't included in the start of the new page.
                            page = Some(Page {global_start_char_pos: i + 1, global_stop_char_pos : i + 1});
                        } else {
                            page = Some(Page {global_start_char_pos: i, global_stop_char_pos : i});
                        }
                    }
                    cur_row = 0;
                } else {
                    // Continue on the same page.
                    cur_row += 1;
                }
                cur_column = 0;
            }
        }

        // Adds the last page to be paginated, can be the first, if it only has one.
        if let Some(mut page_tmp) = page {
            page_tmp.global_stop_char_pos = text_vec.len() - 1;
            pages_vec.push(page_tmp);
        }

        Self {
            curr_page,
            pages_vec,
        }
    }

    fn get_curr_page(&self) -> (usize, &Page) {
        (self.curr_page, &self.pages_vec[self.curr_page])
    }

    fn set_curr_page_num(& mut self, page_num: usize) -> bool {
        if page_num < self.pages_vec.len() {
            self.curr_page = page_num;
            return true;
        }
        false
    }

    fn find_char_pos_in_pages(&self, global_char_pos: usize) -> usize {
        for (page_num, page) in self.pages_vec.iter().enumerate() {
            if    global_char_pos >= page.global_start_char_pos
               && global_char_pos <= page.global_stop_char_pos {
                   return page_num;
               }
        }
        0_usize
    }

    fn next_page(& mut self) -> bool {
        if self.curr_page < self.pages_vec.len() - 1 {
            self.curr_page += 1;
            return true;
        }
        false
    }

    fn prev_page(& mut self) -> bool {
        if self.curr_page > 0 {
            self.curr_page -= 1;
            return true;
        }
        false
    }

    // This is for search.

}

struct Word {
    start: usize,
    _end: usize,
    middle_start: usize,
    _middle_end: usize
}

trait Inside {
    fn is_inside_word_first_half(&self, index: usize) -> bool;

}

impl Inside for Vec<Word> {
    fn is_inside_word_first_half(&self, index: usize) -> bool {
        for word in self {
            if    index >= word.start
               && index <= word.middle_start {
                // Inside   
                return true;
               }
        }
        false
    }    
}

fn print_fp(p_buf: &Vec<char>) {
    // Find the start and end indices of the words in the String and corrects for a sequence of white spaces or tabs.
    let mut words_index: Vec<Word> = Vec::new();
    let mut flag_inside_word = false;
    let mut start = 0 ;
    for (i, c) in p_buf.iter().enumerate() {
        if c.is_alphanumeric() {
            if !flag_inside_word {
                flag_inside_word = true;
                start = i;
            }
        } else if flag_inside_word {
            flag_inside_word = false;
            let end = i;
            let middle_start;
            let middle_end;
            // If word starts with a number, all the word will be bold :-)
            if p_buf[start].is_numeric() {
                (middle_start, middle_end) = (end, end);
            } else {
                (middle_start, middle_end) = calc_middle_start_end_point(start, end);
            }
            words_index.push(Word {start, _end: end, middle_start, _middle_end: middle_end});
        }
    }

    execute!(stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
    let _ = execute!(stdout(), MoveTo(0, 0));

    // Prints in bold and normal, the text on the terminal.
    for (i, c) in p_buf.iter().enumerate() {
        if *c == '\n' {
            let (_col, row) = position().unwrap();
            execute!(stdout(), MoveTo(0, row + 1), SetColors(Colors::new(Green, COLOR_REAL_BLACK)) ).unwrap();
        } else if words_index.is_inside_word_first_half(i) {
            execute!(stdout(), SetColors(Colors::new(Green, COLOR_REAL_BLACK)), Print(&(*c.to_string()).bold()) ).unwrap();
        } else if *c != '\n' {
            execute!(stdout(), SetColors(Colors::new(Green, COLOR_REAL_BLACK)), Print( &(*c.to_string())) ).unwrap();
        } else {
            let (_col, row) = position().unwrap();
            execute!(stdout(), MoveTo(0, row + 1), SetColors(Colors::new(Green, COLOR_REAL_BLACK)) ).unwrap();
        }
    }

}

/// Returns (middle_start, middle_end)
fn calc_middle_start_end_point(word_start: usize, word_end: usize) -> (usize, usize) {
    let len = word_end - word_start;    
    // É Impar?
    let is_odd = len % 2 != 0; 
    let middle_f32 = len as f32 / 2.0;
    let _exact_value = middle_f32 % 1.0;

    let middle_start;
    let middle_end;
    if len >= 5 && is_odd {
        middle_start = word_start + middle_f32 as usize;
        middle_end = word_start + middle_f32 as usize + 1;
    } else if len == 1 {
        middle_start = word_start;
        middle_end = word_start;
    } else {
        middle_start = word_start + middle_f32 as usize - 1;
        middle_end = word_start + middle_f32 as usize;
    }

    (middle_start, middle_end)
}
