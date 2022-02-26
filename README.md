# jtalk_warp

jtalk_wrap is a command-line tool converting kanji to hiragana by wrapping API from [j-talk](https://j-talk.com).

## Features

-   Convert sentences containing kanji into hiragana
-   Convert raw result from j-talk to json
-   Login to your j-talk account _optional_
-   Remap input characters by providing remapping file
    useful when you want to convert **Chinese hanzi** to **Japanese kanji**
    `學`->`学` `场`->`場`

## Usage

```
USAGE:
    jtalk [OPTIONS] <TEXT>

ARGS:
    <TEXT>    Your input text

OPTIONS:
        --cookie-file <PATH>    File for saving cookies
    -h, --help                  Print help information
    -r, --remap <PATH>          JSON file for char remapping
        --remember              Remember option in j-talk login
    -V, --version               Print version information
```
