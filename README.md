This is a clone of the TikTok Timewarp filter - example [here](https://www.youtube.com/watch?v=Pi2MaPZLFcc).

Works on Windows only for now.

To make it work on other OSes, you'd need to interface with the webcam API and create a window to draw pixels on.

`winit` is used to create a windows and [ESCAPI](https://github.com/jarikomppa/escapi) is used to capture the camera output.

To run, 

```sh
cargo run
```.

Make sure you only have 1 webcam enabled!
