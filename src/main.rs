#![feature(let_chains)]
use clap::{crate_version, Arg, Command};
use colored::*;
use crossbeam::queue::ArrayQueue;
use dashmap::DashMap;
use home_dir::HomeDirExt;
use infer;
use leptess::LepTess;
use rayon::prelude::*;
use std::{path::PathBuf, sync::Arc, fs::{File, self}};
use walkdir::WalkDir;
fn main() {
    //the arguments passed to the program
    let matches = Command::new("Meme Finder")
        .author("mwaitzman, mwaitzman@outlook.com")
        .version(crate_version!())
        .about("Finds locally-saved memes via their meme text using OCR")
        .arg(
            Arg::new("folders to search")
                .id("dirs")
                .long("dir")
                .alias("directory")
                .short('d')
                .takes_value(true)
                .multiple_values(true)
                .required(true),
        )
        .arg(
            //TODO: remove this arg and just make the last argument passed to the program be the target text, but how?
            Arg::new("text to search for")
                .id("target_text")
                .long("text")
                .short('t')
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let raw_dirs = matches
        .values_of("dirs")
        .expect("no directories to search through were specified!")
        .into_iter();

    // the folders we're gonna be searching in
    let dirs;
    // The `home-dir` crate doesn't support Windows currently, so we only use it on the Unix target. TODO: write (and submit to upstream) a patch to add Windows support
    #[cfg(target_family = "unix")]
    {
        dirs = raw_dirs
            .map(|path| match path.expand_home() {
                Ok(expanded_path) => expanded_path,
                Err(_) => PathBuf::from(path),
            })
            .collect::<Vec<_>>();
    }
    #[cfg(not(target_family = "unix"))]
    {
        dirs = raw_dirs.collect::<Vec<_>>();
    }

    //the text we're searching for
    //TODO: allow optionally specifying text as regex instead of as a literal string
    let mut target_text = matches
        .value_of("target_text")
        .expect(&"no target text specified!".red())
        .to_string();
    //for case-insensitive search
    //TODO: allow specifying the case sensitivity instead of hardcoding as insensitive
    target_text.make_ascii_lowercase();
    let target_text = target_text;

    //all files (and files from symlinks) found in the folder
    //NOTE: the chosen initial capacity is assuming that the directories chosen contain about 64 images, and hence this'll avoid the initial allocations (0, 4, 8, 16, 32, 64), while still being small enough that in the event of there being less, it won't really matter because it's not wasting much space. However, this number basically came out of my ass - it has no real data backing it up at the moment
    // let mut files = Vec::with_capacity(64);

    //TODO: use something better than raw `println!()`-ing (e.g. a proper logging crate), or at least use colors and descriptors tags/whatever the "[info]", "[warning]", etc. that CLIs typically print is called.
    //TODO: gate this stuff behind a `--verbose`/`-v` command line switch
    println!("starting directory traversal");

    // (recursively) iterate through the specified directories, adding all image files to the list of files to search
    //NOTE: this iterator isn't parallelized because of unmet trait bounds, hence we're doing as little here as possible and doing the rest in a parallel iterator instead
    let mut files: Vec<Option<PathBuf>> = dirs
        .iter()
        .map(|dir| {
            //NOTE: this iterator isn't parallelized because of unmet trait bounds, hence we're doing as little here as possible and doing the rest in a parallel iterator instead
            WalkDir::new(dir).into_iter().map(|entry| {
                let entry = entry.unwrap();
                //if the entry's a file, add it to the list of files to validate
                if entry.file_type().is_file() {
                    Some(entry.into_path())
                }
                //if the entry's a symlink, add the file it points to the list of files to validate
                else if entry.path().is_symlink() {
                    let entry = std::fs::read_link(entry.path()).unwrap();
                    Some(entry)
                // if it's a directory, don't add it, obviously
                } else {
                    None
                }
            })
        })
        .flatten()
        .filter(|entry| entry.is_some())
        .collect();

    //do the rest of the work here, in parallel
    let files: Vec<PathBuf> = files
        .par_iter_mut()
        //check which files are able to be OCR'd, and mark each accordingly
        .map(|item| {
            //safe because the previous filter ensured that every item is a `Some`
            let file = unsafe { item.take().unwrap_unchecked() };
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
        .map(|item| unsafe { item.unwrap_unchecked() })
        .collect();
    println!(
        "Directory traversal complete!\nSearchable files found: {}",
        &files.len().to_string().bold()
    );

    //the number of threads/CPU cores the program'll be parallelized across
    let thread_count = rayon::current_num_threads();
    println!("using {thread_count} threads");

    //the pool of OCR reader structs; one per thread
    let readers = ArrayQueue::new(thread_count);
    for _ in 0..thread_count {
        //guaranteed to be safe because we're pushing exactly {ArrayQueue's length} times, to an originally empty queue
        //CHECK: is there actually a point to doing the `unwrap_unchecked` vs just not unwrapping at all?
        unsafe {
            //CHECK: what's the point of the data_path parameter in `LepTess::new()`? - is it for non-bundled training data??
            readers
                .push(LepTess::new(None, "eng").unwrap())
                .unwrap_unchecked();
        }
    }

    // try to load the cache file (may not exist)
        let cache_file_path = match dirs::cache_dir() {
            Some(mut path) => {
                path.push("meme_finder/image_cache.ron");
                path
            }
            None => PathBuf::from("/.cache/meme_finder/image_cache.ron"),
        };

        //NOTE: doesn't have to be mutable because of the library's design
        let image_cache: Arc<DashMap<u64, String>>
        = if let Ok(cache_file) = File::open(&cache_file_path)
                && let Ok(deserialized_map) = ron::de::from_reader(cache_file) {
                    Arc::new(deserialized_map)
                }
                else {
                    Arc::new(DashMap::new())
                };

    //iterate in parallel over each of the added files, searching each of them for a match
    println!("starting search...");
    files.par_iter().for_each(|file| {
        //check cache for this file
            let file_contents = std::fs::read(&file).unwrap();
            //CHECK: is the quality of this good enough? Should I really be using a 128 or higher bit hash?
            let file_hash = seahash::hash(&file_contents);

            let mut found_text;

            if let Some(content) = image_cache.get(&file_hash) {
                found_text = content.value().to_owned();
                //NOTE: dropping ASAP so as to not cause a deadlock, as the documentation warns can happen if the `get` function is called when holding a mutable reference into the map (although that might be only if there is already a reference to this SAME key (but it's best not to risk it just in case it doesn't just mean that))
                drop(content);

                //TODO: refactor to not have this code duplicated, maybe
                // case-insensitive matching (default for now)
                    found_text.make_ascii_lowercase();
                    let found_text = found_text;
                //TODO: match via regex instead if regex command line switch (TODO) is passed
                if found_text.contains(&target_text) {
                    println!(
                        "found match in file {}",
                        file.as_path().to_str().unwrap().yellow()
                    );
                    //TODO: display image in the terminal if it supports the Kitty Image Protocol
                    /*
                    https://sw.kovidgoyal.net/kitty/graphics-protocol/#querying-support-and-available-transmission-mediums
                    */
                    //TODO: check if match_limit (TODO) reached, terminate if so
                }
            }
        // we evidently haven't cached the file, so let's OCR it
            else {
                //the OCR reader for this file
                //SAFE: guaranteed to be an `Ok()`, because this  doesn't run until there's an item in the ArrayQueue, hence the returned value can't be none
                let mut lt = unsafe { readers.pop().unwrap_unchecked() };

                //load the image file into the reader, and scan it
                    match lt.set_image(&file) {
                        Ok(_) => {
                            found_text = lt.get_utf8_text().unwrap_or("".to_string());

                            //add file to cache
                                let file_text = found_text.clone();
                                image_cache.insert(file_hash, file_text);

                            //case-insensitive matching (default for now)
                                found_text.make_ascii_lowercase();
                                //CHECK: is this "`unmut`'ing" actually zero-cost? I assume it is...
                                let found_text = found_text;
                            //TODO: match via regex instead if regex command line switch (TODO) is passed
                            if found_text.contains(&target_text) {
                                println!(
                                    "found match in file {}",
                                    file.as_path().to_str().unwrap().yellow()
                                );
                                //TODO: display image in the terminal if it supports the Kitty Image Protocol
                                /*
                                https://sw.kovidgoyal.net/kitty/graphics-protocol/#querying-support-and-available-transmission-mediums
                                */
                                //TODO: check if match_limit (TODO) reached, terminate if so
                            }
                        }
                        //TODO: check if the error type is something we care about; handle it if so
                        Err(e) => println!(
                            "WARNING: OCR could not read file ({path:#?}) due to error {e}",
                            path = &file
                        ),
                    }
                //free up the reader for future use
                //guaranteed to be safe because we're just putting it back in its spot
                unsafe {
                    readers.push(lt).unwrap_unchecked();
                }
            }
    });
    //kinda stupid that I have to do it like this... Is there a better way?
    println!("{}", "search complete!".green());

    // write the cache to disk
        //ensure that the folder that it's in exists first to prevent an error (panic, really)
            let full_folder_path = &cache_file_path.parent().unwrap();
            fs::create_dir_all(full_folder_path).unwrap();
        let cache_file = File::create(&cache_file_path).expect("failed to create cache file!");
        ron::ser::to_writer(cache_file, &*image_cache).expect("failed to create serializer");
        println!("serialized cache to {:#?}", &cache_file_path)
}