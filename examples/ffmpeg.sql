
'
import ffmpeg

#.output(str(frames_dir / "%04d.jpeg"))
def frames(video):
  proc = (
      ffmpeg.input("pipe:")
      .output("pipe:", format="rawvideo", pix_fmt="rgb24")
      .run_async(pipe_stdin=True, capture_stdout=True)
  proc.stdin.write(video)
  proc.stdin.close()
  proc.wait()'