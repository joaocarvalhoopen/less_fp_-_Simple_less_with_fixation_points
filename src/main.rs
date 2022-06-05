// Project: less_fp - Simple less with fixation points.
// Author: João Nuno Carvalho
// Data: 2022.06.04
// Description: This simple program reads a text file, paginate it and shows
//              it with fixation points in bold. In principal they are
//              supposed to allow you to read faster.
//              Use Esc to quit.
//              Use 'q' to prev_page.
//              Use 'a' to next_page.
//              Use '/' to search for a string.
//              Use '/' to search + Enter key to exit search mode. 
//              Use 'p' to prev found string.
//              Use 'n' to next found string.
//              Use mouse or keyboard for terminal resize.
//              I tested it under Linux, maybe it works under Windows. <br>
//
// License: MIT Open Source license.
//
// Have fun!
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

const COLOR_REAL_BLACK: Color = Rgb {r: 0, g: 0, b: 0};

/// This simple program reads a text file, paginate it and shows it with
/// fixation points in bold. In principal they are supposed to allow you
/// to read faster.
/// Use Esc to quit.
/// Use 'q' to prev_page.
/// Use 'a' to next_page.
/// Use '/' to search for a string.
/// Use '/' to search + Enter key to exit search mode. 
/// Use 'p' to prev found string.
/// Use 'n' to next found string.
/// Use mouse or keyboard for terminal resize.
///
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Text file
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    file: Option<PathBuf>,

    // TODO:
    // Read arguments from command line : color, page, Search pattern and next and previous.
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

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let mut pages_vec = PageVec::paginate(text_vec, terminal::size());
    let (_page_num, page) = pages_vec.get_curr_page();
    let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
    let search_opt: Option<Search> = None;
    let search_string = "";
    print_fp(&text_vec_page, &search_opt, page.global_start_char_pos,
        &SearchMode::NotInMode, search_string);

    if let Err(e) = print_events(text_vec, & mut pages_vec) {
        println!("Error: {:?}\r", e);
    }

    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()

}

struct TextPos {
    start_pos: usize,
    end_pos: usize,
}

struct Search {
    curr_pos: usize,
    text_pos_vec: Vec<TextPos>,
}

impl Search {
    fn find(global_text: &Vec<char>, search_string: &str) -> Option<Self> {
        let ocurrencies = global_text.find_str_all(search_string);
        if ocurrencies.is_empty() {
            return None;
        }
        let mut text_pos_vec: Vec<TextPos> = Vec::new();
        for start_pos in ocurrencies {
            text_pos_vec.push(
                TextPos { start_pos, end_pos: start_pos + search_string.len() - 1});
        }
        Some(
            Search {
                curr_pos: 0,
                text_pos_vec
                }
            )
    }

    fn get_curr_pos(&self) -> (usize, &TextPos) {
        (self.curr_pos, &self.text_pos_vec[self.curr_pos])
    }

    fn next_pos(& mut self) -> bool {
        if self.curr_pos < self.text_pos_vec.len() - 1 {
            self.curr_pos += 1;
            return true;
        }
        false
    }

    fn prev_pos(& mut self) -> bool {
        if self.curr_pos > 0 {
            self.curr_pos -= 1;
            return true;
        }
        false
    }

    // In here I have to calculate the forward distance from the current position
    // for all occurrences and get the minimal value.
    // If it didn't find, it goes to the first one.
    fn find_next_nearest_pos(& mut self, text_vec: &Vec<char>, global_curr_page_start_pos: usize) -> usize {
        let mut lowest_distance = text_vec.len() as i32;
        let mut last_word_index = 0_usize;

        // Search's next in front of current start of current page start position, in reverse, from the end to the current position.
        for (word_index, text_pos) in self.text_pos_vec.iter().rev().enumerate() {
           let delta = text_pos.start_pos as i32 - global_curr_page_start_pos as i32;
           if delta >= 0 && delta < lowest_distance {
               lowest_distance = delta;
               last_word_index = self.text_pos_vec.len() - 1 - word_index;
           }
        }

        if lowest_distance < text_vec.len() as i32 {
            // Found word occurrence position in front of the start of current page.
            self.curr_pos = last_word_index;
            self.text_pos_vec[last_word_index].start_pos
        } else {
            // Returns the global position of first occurrence in the text file.
            self.curr_pos = 0;
            self.text_pos_vec[0].start_pos
        }           
    }

    fn is_inside_word(& self, global_pos_of_char: usize) -> bool {
        for text_pos in self.text_pos_vec.iter() {
            if    global_pos_of_char >= text_pos.start_pos
               && global_pos_of_char <= text_pos.end_pos {
                   return true;
               }
        }
        false
    }

    fn is_inside_current_word(& self, global_pos_of_char: usize) -> bool {
        if    global_pos_of_char >= self.text_pos_vec[self.curr_pos].start_pos
            && global_pos_of_char <= self.text_pos_vec[self.curr_pos].end_pos {
                return true;
            }
        false
    }

}

enum SearchMode {
    NotInMode,
    EnteringSearchString,
    BrowsingInSearch,
}

fn print_events(text_vec: &Vec<char>, pages_vec: & mut PageVec) -> Result<()> {
    
    let mut search_mode = SearchMode::NotInMode;
    let mut search_string = String::new();
    let mut search_opt: Option<Search> = None;

    loop {
        // Blocking read
        let event = read()?;
       
        // println!("Event: {:?}\r", event);

        // if event == Event::Key(KeyCode::Char('c').into()) {
        //     println!("Cursor position: {:?}\r", position());
        // }

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
            
            print_fp(&text_vec_page, &search_opt, page.global_start_char_pos,
                     &search_mode, &search_string);
        }

        if event == Event::Key(KeyCode::Esc.into()) {
            execute!(stdout(), terminal::Clear(terminal::ClearType::All), MoveTo(0, 0)).unwrap();
            break;
        }

        match search_mode {
            SearchMode::NotInMode | SearchMode::BrowsingInSearch  => {
                    if event == Event::Key(KeyCode::Char('q').into()) && pages_vec.prev_page(){
                        let (_page_num, page) = pages_vec.get_curr_page();        
                        let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
                        print_fp(&text_vec_page, &search_opt, page.global_start_char_pos,
                                 &search_mode, &search_string);
                    }
        
                    if event == Event::Key(KeyCode::Char('a').into()) && pages_vec.next_page(){
                        let (_page_num, page) = pages_vec.get_curr_page();        
                        let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
                        print_fp(&text_vec_page, &search_opt, page.global_start_char_pos,
                                 &search_mode, &search_string);
                    }

                    // Enter in search mode.
                    if event == Event::Key(KeyCode::Char('/').into()){
                        search_string.clear();
                        search_mode = SearchMode::EnteringSearchString;
                        let (_page_num, page) = pages_vec.get_curr_page();        
                        let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
                        print_fp(&text_vec_page, &search_opt, page.global_start_char_pos,
                                 &search_mode, &search_string);
                    }

                    if let SearchMode::BrowsingInSearch = search_mode {
                        if let Some(ref mut search_tmp) = search_opt {
                            
                            // We jump to the prev or next word find occurrence.
                            if   (event == Event::Key(KeyCode::Char('p').into()) && search_tmp.prev_pos())
                              || (event == Event::Key(KeyCode::Char('n').into()) && search_tmp.next_pos()) {
                                
                                // The prev_pos() or the next_pos() as already been made, so we do a get current position.
                                let (_ocurr_index, TextPos { start_pos, end_pos: _ }) = search_tmp.get_curr_pos();                                                                   
                                let page_num =  pages_vec.find_char_pos_in_pages(*start_pos);
                                pages_vec.set_curr_page_num(page_num);

                                let (_page_num, page) = pages_vec.get_curr_page();                                    
                                let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
                                print_fp(&text_vec_page, &search_opt, page.global_start_char_pos,
                                        &search_mode, &search_string);

                            }
                        }
                    }
                },
            SearchMode::EnteringSearchString => {
                    // Exit search mode.
                    match event{
                        Event::Key(key_event) => {
                                if key_event.code == KeyCode::Enter {
                                    if !search_string.is_empty() {
                                        // Do the search in the text.
                                        
                                        // TODO: Possible error not found by the compiler, if we make the next line
                                        // "if let Some(ref search_tmp)"
                                        // and comment the line a few lines below "search_opt = Some(search_tmp);" 
                                        if let Some(mut search_tmp) = Search::find(text_vec, &search_string) {
                                            search_mode = SearchMode::BrowsingInSearch;
                                            // Go to the page and update the screen.
                                            let (_page_num, page) = pages_vec.get_curr_page();
                                            let search_next_pos = search_tmp.find_next_nearest_pos( &text_vec, page.global_start_char_pos);
                                            let page_num = pages_vec.find_char_pos_in_pages(search_next_pos);
                                            search_opt = Some(search_tmp);
                                            pages_vec.set_curr_page_num(page_num);
                                            let (_page_num, page) = pages_vec.get_curr_page();
                                            let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
                                            print_fp(&text_vec_page, &search_opt, page.global_start_char_pos,
                                                     &search_mode, &search_string);
                                            continue;
                                        } else {
                                            search_opt = None;
                                            search_mode = SearchMode::NotInMode;   
                                        }
                                    } else {
                                        search_opt = None;
                                        search_mode = SearchMode::NotInMode;
                                    }
                                } else if key_event.code == KeyCode::Backspace {
                                    search_string.pop();                                 
                                } else if let KeyCode::Char(c) = key_event.code {
                                    search_string.push(c);
                                }
                                let (_page_num, page) = pages_vec.get_curr_page();        
                                let text_vec_page = text_vec[page.global_start_char_pos..=page.global_stop_char_pos].to_vec();
                                print_fp(&text_vec_page, &search_opt, page.global_start_char_pos,
                                         &search_mode, &search_string);
                            },
                        // Events processed before this point.
                        Event::Mouse(_) => (),
                        Event::Resize(_, _) => (),
                    };
                    
                    
                },
            
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

fn print_fp(p_buf: &Vec<char>, search_opt: &Option<Search>, global_start_pos: usize,
            search_mode: &SearchMode, search_string: &str) {
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
        let mut flag_search_inside_word = false;
        let mut flag_search_inside_current_word = false;
        if let Some(ref search_tmp ) = search_opt {
            flag_search_inside_word = search_tmp.is_inside_word(global_start_pos + i); 
            flag_search_inside_current_word = search_tmp.is_inside_current_word(global_start_pos + i); 
        }

        if *c == '\n' {
            let (_col, row) = position().unwrap();
            execute!(stdout(), MoveTo(0, row + 1), SetColors(Colors::new(Green, COLOR_REAL_BLACK)) ).unwrap();
        } else if words_index.is_inside_word_first_half(i) {
            if flag_search_inside_current_word {
                execute!(stdout(), SetColors(Colors::new(DarkGrey, White)), Print(&(*c.to_string()).bold()) ).unwrap();
            } else if flag_search_inside_word {
                execute!(stdout(), SetColors(Colors::new(Blue, White)), Print(&(*c.to_string())) ).unwrap();
            } else {
                execute!(stdout(), SetColors(Colors::new(Green, COLOR_REAL_BLACK)), Print(&(*c.to_string()).bold()) ).unwrap();
            }
        } else if *c != '\n' {

            if flag_search_inside_current_word {
                execute!(stdout(), SetColors(Colors::new(DarkGrey, White)), Print(&(*c.to_string()).bold()) ).unwrap();
            } else if flag_search_inside_word {
                execute!(stdout(), SetColors(Colors::new(Blue, White)), Print( &(*c.to_string())) ).unwrap();
            } else {
                execute!(stdout(), SetColors(Colors::new(Green, COLOR_REAL_BLACK)), Print( &(*c.to_string())) ).unwrap();
            }
        } else {
            let (_col, row) = position().unwrap();
            execute!(stdout(), MoveTo(0, row + 1), SetColors(Colors::new(Green, COLOR_REAL_BLACK)) ).unwrap();
        }
    }

    match search_mode {
        SearchMode::NotInMode => (),
        SearchMode::EnteringSearchString => {
            let (_len_col, len_row) = terminal::size().unwrap();
            let string_out = "/ ".to_string() + search_string;
            execute!(stdout(), MoveTo(0, len_row - 1), SetColors(Colors::new(White, DarkBlue)), Print( &(string_out)) ).unwrap();
            execute!(stdout(), SetColors(Colors::new(Green, COLOR_REAL_BLACK)) ).unwrap();
            },
        SearchMode::BrowsingInSearch => (),
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
