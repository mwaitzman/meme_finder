use std::path::PathBuf;
use leptess::LepTess;
use rayon::prelude::*;
use walkdir::WalkDir;
use crossbeam::queue::ArrayQueue;
use infer;

fn main() {
    //the arguments passed to the program
    //TODO: migrate to clap (or similar) for command line args for much more customizability and robustness
    let mut args = std::env::args();

    //the folder we're searching in
    //TODO: allow specifying multiple destination folders (and maybe also specific files to scan - this might be nearly useless because if specifying specific files manually, may as well search them manually instead)
    let memes_folder = args.nth(1).unwrap();
    //the text we're searching for

    //TODO: allow optionally specifying text as regex instead of as a literal string
    let mut target_text = args.next().unwrap();
    //for case-insensitive search
    //TODO: allow specifying case (in)sensitivity
    target_text.make_ascii_lowercase();
    let target_text = target_text;

    //all files (and files from symlinks) found in the folder
    let mut files = vec![];

    //TODO: use something better than raw `println!()`-ing (e.g. a proper logging crate), or at least use colors and descriptors tags/whatever the "[info]", "[warning]", etc. that CLIs typically print is called.
    //TODO: gate this stuff behind a `--verbose`/`-v` command line switch
    println!("starting directory traversal");

    //(recursively) iterate through the specified directory, adding all image files to the list of files to search
    //NOTE: this iterator isn't parallelized because of unmet trait bounds, hence we're doing as little here as possible and doing the rest in a parallel iterator instead
    WalkDir::new(memes_folder).into_iter().for_each(|entry| {
        let entry = entry.unwrap();
        //if the entry's a file, add it to the list of files to validate
        if entry.file_type().is_file() {
            files.push(Some(entry.into_path()));
        }
        //if the entry's a symlink, add the file it points to the list of files to validate
        else if entry.path().is_symlink() {
            let entry = std::fs::read_link(entry.path()).unwrap();
            files.push(Some(entry));
        }
    });

    //do the rest of the work here, in parallel
    let files: Vec<PathBuf> = files.par_iter_mut()
        //check which files are able to be OCR'd, and mark each accordingly
        .map(|item| {
            //safe because we know it's a `Some`
            let file = unsafe {item.take().unwrap_unchecked()};
                //fetch the file from its path, and check if it's an image file
                //TODO: make sure the file type is actually supported by the OCR engine
                //CHECK: I think the file is already guaranteed to exist, so the `if let` can be replaced with a pure fetching to more clearly indicate this
                if let Ok(Some(filetype)) = infer::get_from_path(&file) {
                    if let infer::MatcherType::Image = filetype.matcher_type() {
                        //file's valid, put it back
                        return Some(file);
                    }
                }
                //this file's invalid, so mark it as to be discarded
                None
        })
        //filter out the files that can't be OCR'd
        .filter(|item| item.is_some())
        //map from all `Some(PathBuf)`s to just `PathBuf`s
        .map(|item| unsafe {item.clone().unwrap_unchecked()})
    .collect();
    println!("Directory traversal complete!\nSearchable files found: {}", &files.len());

    //the number of threads/CPU cores the program'll be parallelized across
    let thread_count = rayon::current_num_threads();
    println!("using {thread_count} threads");

    //the pool of OCR reader structs; one per thread
    let readers = ArrayQueue::new(thread_count);
    for _ in 0..thread_count {
        //guaranteed to be safe because we're pushing exactly {ArrayQueue's length} times, to an originally empty queue
        //CHECK: is there actually a point to doing the `unwrap_unchecked` vs just not unwrapping at all?
        unsafe {
            //CHECK: what's the point of the data_path parameter in `LepTess::new()`?
            readers.push(LepTess::new(None, "eng").unwrap()).unwrap_unchecked();
        }
    }

    //iterate in parallel over each of the added files, searching each of them for a match
    println!("starting search...");
    files.par_iter().for_each(|file| {
        //the OCR reader for this file
        //SAFE: guaranteed to be an `Ok()`, because this  doesn't run until there's an item in the ArrayQueue, hence the returned value can't be none
        let mut lt = unsafe {
            readers.pop().unwrap_unchecked()
        };

        //load the image file into the reader, and scan it
        match lt.set_image(file) {
            Ok(_) => {
                let mut found_text = lt.get_utf8_text().unwrap_or("".to_string());
                found_text.make_ascii_lowercase();
                let found_text = found_text;
                //TODO: match via regex instead if regex command line switch (TODO) is passed
                if found_text.contains(&target_text) {
                    println!("found match in file {}", file.as_path().to_str().unwrap())
                    //TODO: display image in the terminal if it supports the Kitty Image Protocol
                    //TODO: check if match_limit (TODO) reached, terminate if so
                }
            },
            //TODO: check if the error type is something we care about; handle it if so
            Err(_) => ()
        }
        //free up the reader for future use
        //guaranteed to be safe because we're just putting it back in its spot
        unsafe { readers.push(lt).unwrap_unchecked();}
    });
    println!("search complete!");
}