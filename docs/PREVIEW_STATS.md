# Preview Stats Reference

This doc explains the preview performance overlay shown in the Preview panel.

Each value reflects the most recent preview render request.

- `total`: Total wall-clock time for the render (ms).
- `collect`: Time spent gathering visible layers for the current time (ms).
- `vdec`: Time spent decoding video frames (ms).
- `still`: Time spent loading still images (ms).
- `comp`: Time spent compositing layers into the RGBA canvas (ms).
- `upload`: Time spent preparing the preview buffer for display (ms).
- `gpu`: Time spent uploading the latest preview frame to the native wgpu texture (ms).
- `hit`: Cache hit percentage for frame lookups during this render.
- `layers`: Number of visual layers composited for this render.
