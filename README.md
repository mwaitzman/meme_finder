### usecase:
you have a folder with a bunch (possibly hundreds or even thousands) of memes. You want to find a specific meme, but obviously don't want to spend an hour manually looking for it manually. That's where this program comes in! If you can remember any of the text written on the meme, you can input the folder's location along with the text you remember the meme has, and this program will search in the folder for the meme, printing out any images that contain that text!

Usage:
    run with `<binary name> <path to image directory> <exact text to search for (case-insensitive)> `to iterate through the directory, finding all image(s) containing that text, and displaying their path(s)

upcoming features:
* better output
    * pretty text
        * colored text
        * nicely formatted text
    * progress display
    * display preview of image in terminal if the terminal supports the Kitty Image Protocol
* better text matching
    * allow case-sensitive search (currently hardcoded as case-insensitive)
    * allow synonym search (maybe, likely won't be be fully accurate)
    * simple typo tolerance (both by you and by the meme)
    * tense tolerance (e.g. "search" will match "searched")
    * allow regex search
* terminate after a specified amount of matches (limit switch)
* allow specifying file creation (and maybe modification) date filters
* speed improvements
    * more parallelism
    * caching of resulting text in a hashmap with the file hash as a key, preserved between sessions for a major speedboost
