Sorts media files based on their file name.

Currently, only detects if it is a TV show or movie based on the presence of SxxExx in the file name.

Improvements:
- Become a MQTT broker so that we can interface with Home assistant and notify when a new show/movie is added or when
  an issue occured while sorting files
- Use async rust
