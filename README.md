Sorts media files based on their file name.

Currently, only detects if it is a TV show or movie based on the presence of SxxExx in the file name.

Improvements:
- Shows:
  - Sometimes, media file names will not contain the show's name, only the season and episode numbers.
    The show name should be retrieved from the folder name or its parents.
    Multiple formats do exist:
    - `<show name>/[Season ]\d\d/Episode \d\d.<ext>`
    - `<show name>/[Season ]\d\d/S\d\dE\d\d.<ext>`
    - `<show name>/S\d\dE\d\d.<ext>`
    Note that `<show name>` probably needs to be normalized.
  - Special episodes sometimes do not contain any `E\d\d` values and are just one of:
    - `<show name>/[Season ]\d\d/<Special episode title>.<ext>`
    - `<show name>/[Season ]\d\d/Specials/<Special episode title>.<ext>`
    - Maybe others, TBC
- Generally, some fallbacks should be put in place: If a movie name cannot be found online, maybe it is a show.
- It would improve the system a lot to keep a small database of the media files that couldn't be sorted.
  With that a web interface would ask if it should be ignored or what name it should use.
- Become a MQTT broker so that we can interface with Home assistant and notify when a new show/movie is added or when
  an issue occurred while sorting files
- Use async rust
