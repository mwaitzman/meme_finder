usecase: you have a folder with a bunch (possibly hundreds or even thousands) of memes. You want to find a specific meme, but obviously don't want to spend an hour manually looking for it manually. That's where this program comes in! If you can remember any of the text written on the meme, you can input the folder's location along with the text you remember the meme has, and this program will search in the folder for the meme, printing out any images that contain that text!

MVP:
    run with `<binary name> <path to directory> <exact text to search for> `to iterate through the directory, finding all image(s) matching the text, and displaying their path(s)

upcoming features:
    * display preview of image in terminal if the terminal supports the Kitty Image Protocol
    * allow regex search
    * allow synonym search
    * do as much in parallel as possible
    * terminate after a specified amount of matches (limit switch)