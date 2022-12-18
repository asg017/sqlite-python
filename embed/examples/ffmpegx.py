import ffmpeg

#.output(str(frames_dir / "%04d.jpeg"))

def frames(video):

  proc = (
      ffmpeg
      .input("pipe:", format='rawvideo', pix_fmt='rgb24',  s=f'{950}x{540}')
      .output("pipe:", vframes=1, pix_fmt='yuv420p')
      .run_async(pipe_stdin=True)
  )
  proc.stdin.write(video)
  proc.stdin.close()
  proc.wait()

print(frames(open("flower.webm", "rb")))