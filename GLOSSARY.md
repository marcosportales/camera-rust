# Camera App

A desktop viewfinder for taking photos and recording videos with the computer's webcam, designed for a tiling Wayland compositor.

## Language

**Viewfinder**:
The live image from the camera that fills the window and is what the user frames a Capture with.
_Avoid_: Preview, feed, stream

**Capture**:
A photo or video produced by the app and saved to disk.
_Avoid_: Shot, recording (as a generic term), media

**Photo**:
A Capture consisting of a single still image.
_Avoid_: Picture, snapshot, image

**Video**:
A Capture consisting of a recorded stretch of moving image.
_Avoid_: Clip, recording

**Mode**:
Whether the next Shutter press produces a Photo or a Video. The user switches Mode with two separate option buttons.
_Avoid_: Tab, type

**Shutter**:
The single main button that starts a Capture (and ends it, for a Video).
_Avoid_: Record button, capture button, trigger

**Timer**:
An optional countdown (off, 3s, 10s) between pressing the Shutter and the Capture starting.
_Avoid_: Delay, countdown

**Mirror**:
Horizontal flip of the Viewfinder so it behaves like a mirror.
_Avoid_: Flip, selfie mode

**Resolution**:
The pixel size at which the camera delivers the Viewfinder and Captures.
_Avoid_: Quality, size

**Last-Capture Thumbnail**:
A small image of the most recent Capture, shown as in phone camera apps; activating it opens the folder containing Captures in the system file manager.
_Avoid_: Gallery, preview
