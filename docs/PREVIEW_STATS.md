# Preview Stats Reference

This doc explains the preview performance overlay shown in the Preview panel.

Each value reflects the most recent preview render request.

- `total`: Total wall-clock time for the render (ms).
- `scan`: Time spent scanning tracks, resolving assets, and cache checks (excludes decode and still load) (ms).
- `vdec`: Time spent decoding video frames (ms).
- `seek`: Time spent seeking the source stream (ms).
- `pkt`: Time spent demuxing/decoding packets up to the target frame (ms).
- `xfer`: Time spent transferring GPU frames back to CPU (ms).
- `scale`: Time spent scaling decoded frames to preview size (ms).
- `copy`: Time spent copying the RGBA frame into the preview buffer (ms).
- `hwdec`: Percentage of decoded video frames that used hardware acceleration (per render).
- `still`: Time spent loading still images (ms).
- `comp`: Time spent compositing layers into the RGBA canvas (ms).
- `upload`: Time spent preparing the preview buffer for display (ms).
- `gpu`: Time spent uploading the latest preview frame to the native wgpu texture (ms).
- `hit`: Cache hit percentage for frame lookups during this render.
- `layers`: Number of visual layers composited for this render.

Note: `total` is the only wall-clock timer. The other fields are per-stage durations. `vdec` is the sum of `seek`, `pkt`, `xfer`, `scale`, and `copy`. `hwdec` shows `--` when no video decode ran for the render.
