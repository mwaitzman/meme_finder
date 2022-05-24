![AGPL3_logo](https://www.gnu.org/graphics/agplv3-with-text-162x68.png)

### usecase:
you have a folder with a bunch (possibly hundreds or even thousands) of memes (or other images, for that matter). You want to find a specific meme, but obviously don't want to spend an hour manually looking for it manually. That's where this program comes in! If you can remember any of the text written on the meme, you can input the folder's location along with the text you remember the meme has, and this program will search in the folder for the meme, printing out any images that contain that text!

### Usage:
run with `<binary name> --dir <path to first directory to search through> <path to second directory to search through> <paths to any more directories you want to search through> --text <exact text to search for (case-insensitive)> ` to iterate through the directory, finding all image(s) containing that text, and displaying their path(s)

### Features:
* Comprehensive and comment-first source code documentation
* Terminal image preview (for best results, use a terminal that supports either [Kitty's terminal graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/#terminal-graphics-protocol) (recommended), or [iTerm2's inline image protocol](https://iterm2.com/documentation-images.html), such as [WezTerm](https://github.com/wez/wezterm))
* persistent caching of file text, making runs with duplicated images and/or already cached directories much, *much*, faster
* an exceptionally ethical license ([The GNU Affero General Public License, version 3](https://www.gnu.org/licenses/agpl-3.0.html))
* recursive directory traversal
* much parallelism
* pretty text
* case-insensitive search


### upcoming features:
* better output
    * proper logging
    * ability to run verbosely or quietly
    * more pretty text
        * colored text
        * nicely formatted text
    * progress display
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
    * more optimizations
* sensible, but configurable defaults
    * symlink tolerance (follows symlinks by default, but that can be changed)
    * recursive directory traversal by default, but the descension depth limit can be set to whatever you wish
    * case-insensitive search, but that can be disabled
* non-english text support (currently hardcoded as searching for English text, but the underlying engine (Tesseract/Leptonica) should be able to support much more)
* environment variable handling
* a very well-defined coding style (currently kind of a work in progress, but readability is slowly getting better and better)
