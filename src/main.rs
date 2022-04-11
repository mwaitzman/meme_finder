use leptess::LepTess;
use rayon::prelude::*;
use walkdir::WalkDir;
fn main() {
    //TODO: migrate to clap for command line args for much more customizability and robustness
    let mut args = std::env::args();
    //TODO: allow specifying multiple destination folders (and maybe also specific files to scan - this might be nearly useless because if specifying specific files manually, may as well search them manually instead)
    let memes_folder = args.nth(1).unwrap();
    //TODO: allow specifying text as regex instead of a literal string
    let target_text = args.next().unwrap();
    let mut files = vec![];
    //(recursively) iterate through the specified directory, adding all image files to the list of files to search
    //CHECK: can this be parallelized?
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
    //iterate in parallel over each of the added files, searching each of them for a match
    files.par_iter().for_each(|file| {
        //CHECK: is this hotly-looped allocation bad? Change if so, but how?
        //CHECK: what's the point of the data_path parameter?
        let mut lt = LepTess::new(None, "eng").unwrap();
        match lt.set_image(file) {
            Ok(_) => {
                let found_text = lt.get_utf8_text().unwrap_or("".to_string());
                if found_text.contains(&target_text) {//TODO: match via regex instead if regex command line switch (TODO) is passed
                    println!("found match in file {}", file.as_path().to_str().unwrap())
                    //TODO: display image in the terminal if it supports the Kitty Image Protocol
                    //TODO: check if match_limit (TODO) reached, terminate if so
                }
            }
            Err(_) => () //TODO: check if the error type is something we care about
        }
    });
}
