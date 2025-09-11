## Description

Project made to learn about raycast style rendering with a unique twist: renderer outputs directly to terminal.
Uses operating system API to draw the picture (rendered with characters in terminal using lines), on Windows there is also a working (commented out) support for input, allowing to move and rotate within the map.

## Build

For the camera output compile and run with:

```
cargo run --release
```

The debug version prints only X Y coordinates and camera yaw just for debugging.
