# Use GStreamer for camera capture and recording

One GStreamer pipeline (v4l2src → tee → Viewfinder / encoder + muxer) feeds both the Viewfinder and Video recording, and produces Photos. We chose it over direct V4L2 + an ffmpeg subprocess or a pure-Rust encoder because codec, container and audio muxing come for free; the cost is a system dependency on GStreamer and its plugins.
