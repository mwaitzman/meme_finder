use leptess::LepTess;
use rayon::prelude::*;
use walkdir::WalkDir;
use crossbeam::queue::ArrayQueue;
use infer;
fn main() {
    //TODO: migrate to clap for command line args for much more customizability and robustness
    let mut args = std::env::args();
    //TODO: allow specifying multiple destination folders (and maybe also specific files to scan - this might be nearly useless because if specifying specific files manually, may as well search them manually instead)
    let memes_folder = args.nth(1).unwrap();
    //TODO: allow specifying text as regex instead of a literal string
    let mut target_text = args.next().unwrap();
    target_text.make_ascii_lowercase();
    let target_text = target_text;
    let mut files = vec![];
    //(recursively) iterate through the specified directory, adding all image files to the list of files to search
    //CHECK: can this be parallelized?
    println!("starting directory traversal");
    WalkDir::new(memes_folder).into_iter().for_each(|entry| {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            //TODO: make sure the file type is actually supported by the OCR engine
            if let Ok(Some(filetype)) = infer::get_from_path(&entry.path()) {
                if let infer::MatcherType::Image = filetype.matcher_type() {
                    files.push(entry.into_path());
                }
            }
        }
    });
    println!("Directory traversal complete!\nSearchable files found: {}", &files.len());

    let thread_count = rayon::current_num_threads();
    println!("using {thread_count} threads");

    //CHECK: what's the point of the data_path parameter in `LepTess::new()`?
    let readers = ArrayQueue::new(thread_count);
    for _ in 0..thread_count {
        //guaranteed to be safe because we're pushing exactly {ArrayQueue's length} times, to an originally empty queue
        //CHECK: is there actually a point to doing the `unwrap_unchecked` vs just not unwrapping at all?
        unsafe {
            readers.push(LepTess::new(None, "eng").unwrap()).unwrap_unchecked();
        }
    }

    //iterate in parallel over each of the added files, searching each of them for a match
    println!("starting search...");
    files.par_iter().for_each(|file| {
        
        //guaranteed to be an `Ok()`, because this  doesn't run until there's an item in the ArrayQueue, hence the returned value can't be none
        let mut lt = unsafe {
            readers.pop().unwrap_unchecked()
        };

        match lt.set_image(file) {
            Ok(_) => {
                let mut found_text = lt.get_utf8_text().unwrap_or("".to_string());
                found_text.make_ascii_lowercase();
                let found_text = found_text;
                if found_text.contains(&target_text) {//TODO: match via regex instead if regex command line switch (TODO) is passed
                    println!("found match in file {}", file.as_path().to_str().unwrap())
                    //TODO: display image in the terminal if it supports the Kitty Image Protocol
                    //TODO: check if match_limit (TODO) reached, terminate if so
                }
            }
            Err(_) => () //TODO: check if the error type is something we care about; handle it if so
        }
        //free up the reader for future use
        //guaranteed to be safe because we're just putting it back in its spot
        unsafe { readers.push(lt).unwrap_unchecked();}
    });
    println!("search complete!");
}