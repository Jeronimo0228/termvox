#!/usr/bin/env python3
"""Render a practical pitch demo from a terminal screen recording.

No title cards, no captions, no marketing copy — only the terminal session,
letterboxed and graded for LinkedIn / pitch playback.

Usage:
  /usr/bin/python3.14 scripts/render-linkedin-demo.py \\
    --source "/path/to/recording.mp4" \\
    --out-dir "$HOME/Vídeos/termvox-demo"
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

W4K, H4K = 3840, 2160
W1080, H1080 = 1920, 1080
FPS = 30
# Match TermVox / terminal chrome
PAD = "0x0B0E14"


def run(cmd: list[str]) -> None:
    print("+", " ".join(str(c) for c in cmd[:12]), "..." if len(cmd) > 12 else "", flush=True)
    subprocess.run(cmd, check=True)


def probe_duration(path: Path) -> float:
    out = subprocess.check_output(
        [
            "ffprobe",
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            str(path),
        ],
        text=True,
    ).strip()
    return float(out)


def x264(kind: str) -> list[str]:
    if kind == "4k":
        rate = ["-b:v", "28M", "-maxrate", "36M", "-bufsize", "56M"]
    else:
        rate = ["-b:v", "12M", "-maxrate", "16M", "-bufsize", "24M"]
    return [
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-profile:v",
        "high",
        "-preset",
        "medium",
        *rate,
        "-x264-params",
        "aq-mode=3:aq-strength=1.1",
        "-r",
        str(FPS),
        "-movflags",
        "+faststart",
    ]


def build_vf(width: int, height: int, duration: float, start: float, fade_in: float, fade_out: float) -> str:
    # Practical cut: scale to fit, pad to 16:9, light contrast/sharpen, soft fades.
    # No drawtext / ass / overlays.
    fade_out_start = max(0.0, duration - start - fade_out)
    return (
        f"fps={FPS},"
        f"scale={width}:{height}:force_original_aspect_ratio=decrease:flags=lanczos,"
        f"pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:{PAD},"
        "eq=contrast=1.06:brightness=0.02:saturation=1.05,"
        "unsharp=5:5:0.7:5:5:0.0,"
        f"fade=t=in:st=0:d={fade_in},"
        f"fade=t=out:st={fade_out_start:.3f}:d={fade_out}"
    )


def render(source: Path, out: Path, width: int, height: int, start: float, fade_in: float, fade_out: float) -> None:
    duration = probe_duration(source)
    usable = max(0.5, duration - start)
    vf = build_vf(width, height, duration, start, fade_in, fade_out)
    kind = "4k" if width >= 3000 else "1080"
    cmd = [
        "ffmpeg",
        "-y",
        "-ss",
        f"{start:.3f}",
        "-i",
        str(source),
        "-t",
        f"{usable:.3f}",
        "-vf",
        vf,
        "-an",
        *x264(kind),
        str(out),
    ]
    run(cmd)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path.home() / "Vídeos" / "termvox-demo",
    )
    parser.add_argument(
        "--start",
        type=float,
        default=1.2,
        help="Seconds to trim from the start (skip idle/typing lead-in)",
    )
    parser.add_argument("--fade-in", type=float, default=0.35)
    parser.add_argument("--fade-out", type=float, default=0.55)
    args = parser.parse_args()

    if not args.source.exists():
        print(f"source not found: {args.source}", file=sys.stderr)
        return 1
    if not shutil.which("ffmpeg"):
        print("ffmpeg required", file=sys.stderr)
        return 1

    args.out_dir.mkdir(parents=True, exist_ok=True)
    master = args.out_dir / "termvox-linkedin-4k.mp4"
    linkedin = args.out_dir / "termvox-linkedin-1080p.mp4"

    print(f"source: {args.source}")
    print(f"duration: {probe_duration(args.source):.2f}s  start trim: {args.start:.2f}s")
    print("mode: practical terminal demo (no on-screen text)")

    render(args.source, master, W4K, H4K, args.start, args.fade_in, args.fade_out)
    render(args.source, linkedin, W1080, H1080, args.start, args.fade_in, args.fade_out)

    print()
    print("Wrote:")
    print(f"  {master}")
    print(f"  {linkedin}")
    print("Upload the 1080p file for LinkedIn / pitch decks.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
