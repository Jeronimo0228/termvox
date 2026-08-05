#!/usr/bin/env python3
"""midudev-style practical TermVox demo for X / LinkedIn.

Research takeaways (X @midudev OSS/feature clips, Aug 2026):
  - 12–20s of pure product motion (no title cards, no voiceover text in-frame)
  - Hook + bullets live in the post copy, not burned into the video
  - Jump straight to the magic; skip long setup
  - Tight framing, dark UI, something changes every second
  - End on the wow result

This script:
  1) ~3.5s ultra-fast terminal flash (init/doctor/shell) — optional
  2) Jump-cuts from your screen recording: listening → prompt → agent
  3) Soft zoom punches + ANSI glitch cover + full-width 16:9

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
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

W, H = 3840, 2160
FPS = 30
BG = (10, 12, 16)
FG = (230, 236, 242)
DIM = (110, 122, 136)
TEAL = (45, 212, 191)
GREEN = (74, 222, 128)
PANEL = (18, 22, 28)
CURSOR = (45, 212, 191)

FONT_REG = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Regular.otf"
FONT_MED = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Medium.otf"

# Source 1920x924 — cover ANSI crumbs (source-absolute times)
GLITCH_BOXES_SRC = [
    (560, 430, 620, 90, "0x0A0C10", 10.5, 34.5),
    (220, 700, 900, 50, "0x141820", 10.5, 40.0),
]

# Jump-cut storyboard on SOURCE timeline (tuned for 2026-08-04 23-39-46 take)
# listening / shell ready → prompt → agent working → interactive scope
SEGMENTS = [
    # (start, end, zoom, label)
    (2.0, 7.5, 1.0, "shell_ready"),
    (14.0, 20.5, 1.08, "listening_prompt"),  # zoom into action
    (28.0, 38.0, 1.04, "agent_work"),
    (48.0, 57.5, 1.0, "result_ui"),
]


def font(path: str, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(path, size=size)


def run(cmd: list[str]) -> None:
    print("+", " ".join(str(c) for c in cmd[:16]), "..." if len(cmd) > 16 else "", flush=True)
    subprocess.run(cmd, check=True)


def probe_duration(path: Path) -> float:
    return float(
        subprocess.check_output(
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
    )


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


class TerminalScene:
    def __init__(self) -> None:
        self.lines: list[tuple[str, tuple[int, int, int]]] = []
        self.prompt_cwd = "~/proyecto"
        self.partial = ""
        self.show_cursor = True
        self.f_reg = font(FONT_REG, 48)
        self.f_med = font(FONT_MED, 48)
        self.pad_x = 110
        self.pad_y = 90
        self.line_h = 64
        self.max_visible = 24

    def add(self, text: str, color: tuple[int, int, int] = FG) -> None:
        self.lines.append((text, color))

    def render(self) -> Image.Image:
        img = Image.new("RGB", (W, H), BG)
        draw = ImageDraw.Draw(img)
        draw.rectangle([0, 0, W, 52], fill=PANEL)
        for i, col in enumerate(((255, 95, 87), (255, 189, 46), (39, 201, 63))):
            x = 28 + i * 34
            draw.ellipse([x, 16, x + 20, 36], fill=col)
        draw.text((140, 12), f"termvox — {self.prompt_cwd}", font=font(FONT_MED, 30), fill=DIM)

        visible = self.lines[-self.max_visible :]
        y = self.pad_y + 30
        for text, color in visible:
            draw.text((self.pad_x, y), text, font=self.f_reg, fill=color)
            y += self.line_h

        prompt = f"{self.prompt_cwd} ❯ "
        draw.text((self.pad_x, y), prompt, font=self.f_med, fill=TEAL)
        px = self.pad_x + int(draw.textlength(prompt, font=self.f_med))
        draw.text((px, y), self.partial, font=self.f_reg, fill=FG)
        if self.show_cursor:
            cx = px + int(draw.textlength(self.partial, font=self.f_reg))
            draw.rectangle([cx + 4, y + 10, cx + 30, y + 52], fill=CURSOR)
        return img


def build_flash_animation() -> list[Image.Image]:
    """~3.5s midu-style cold open: three commands, no lingering."""
    scene = TerminalScene()
    frames: list[Image.Image] = []

    def type_fast(cmd: str) -> None:
        scene.partial = ""
        n = max(6, int(len(cmd) * 0.55))
        for i in range(n + 1):
            scene.partial = cmd[: int(len(cmd) * i / n)]
            scene.show_cursor = i % 2 == 0
            frames.append(scene.render())
        scene.add(f"{scene.prompt_cwd} ❯ {cmd}", FG)
        scene.partial = ""
        scene.show_cursor = False
        frames.append(scene.render())

    def lines(items: list[tuple[str, tuple[int, int, int]]], hold: int = 1) -> None:
        for text, color in items:
            scene.add(text, color)
            for _ in range(hold):
                frames.append(scene.render())

    type_fast("termvox init --preset opencode")
    lines([("wrote termvox.toml", TEAL)], hold=2)
    type_fast("termvox doctor")
    lines(
        [
            ("[ok] microphone   ready", GREEN),
            ("[ok] whisper      ggml-base", GREEN),
            ("[ok] opencode     authenticated", GREEN),
        ],
        hold=2,
    )
    type_fast("termvox shell --agent opencode --fresh")
    lines([("mic bar · F8 / Ctrl+Space", TEAL)], hold=3)
    for i in range(8):
        frames.append(Image.blend(scene.render(), Image.new("RGB", (W, H), (0, 0, 0)), i / 7))
    return frames


def encode_frames(frames: list[Image.Image], out_mp4: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="termvox-flash-") as tmp:
        tmp_path = Path(tmp)
        for i, frame in enumerate(frames):
            frame.save(tmp_path / f"frame_{i:05d}.png")
        run(
            [
                "ffmpeg",
                "-y",
                "-framerate",
                str(FPS),
                "-i",
                str(tmp_path / "frame_%05d.png"),
                "-frames:v",
                str(len(frames)),
                *x264("4k"),
                str(out_mp4),
            ]
        )


def glitch_filters(src_start: float, src_end: float) -> str:
    """drawbox enables relative to segment start (after -ss)."""
    parts = []
    for x, y, w, h, color, t0, t1 in GLITCH_BOXES_SRC:
        # overlap of [t0,t1] with [src_start, src_end], shifted to local t
        a = max(t0, src_start) - src_start
        b = min(t1, src_end) - src_start
        if b <= 0 or a >= (src_end - src_start):
            continue
        parts.append(
            f"drawbox=x={x}:y={y}:w={w}:h={h}:color={color}@1:t=fill:enable='between(t,{a:.2f},{b:.2f})'"
        )
    return ",".join(parts)


def render_segment(source: Path, out: Path, start: float, end: float, zoom: float) -> None:
    dur = max(0.2, end - start)
    g = glitch_filters(start, end)
    prefix = f"{g}," if g else ""
    # Fit full width; pad with TUI black. Optional soft zoom (center).
    if zoom > 1.001:
        # zoompan after pad — subtle punch
        zexpr = f"min({zoom:.3f},1+0.0008*on)"
        vf = (
            f"{prefix}"
            f"fps={FPS},"
            f"scale={W}:{H}:force_original_aspect_ratio=decrease:flags=lanczos,"
            f"pad={W}:{H}:(ow-iw)/2:(oh-ih)/2:0x0A0C10,"
            f"zoompan=z='{zexpr}':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d=1:s={W}x{H}:fps={FPS},"
            "eq=contrast=1.06:brightness=0.02:saturation=1.05,"
            "unsharp=5:5:0.7:5:5:0.0"
        )
    else:
        vf = (
            f"{prefix}"
            f"fps={FPS},"
            f"scale={W}:{H}:force_original_aspect_ratio=decrease:flags=lanczos,"
            f"pad={W}:{H}:(ow-iw)/2:(oh-ih)/2:0x0A0C10,"
            "eq=contrast=1.06:brightness=0.02:saturation=1.05,"
            "unsharp=5:5:0.7:5:5:0.0"
        )
    run(
        [
            "ffmpeg",
            "-y",
            "-ss",
            f"{start:.3f}",
            "-i",
            str(source),
            "-t",
            f"{dur:.3f}",
            "-vf",
            vf,
            "-an",
            *x264("4k"),
            str(out),
        ]
    )


def concat(parts: list[Path], out_mp4: Path) -> None:
    lst = out_mp4.with_suffix(".txt")
    lst.write_text("".join(f"file '{p.resolve()}'\n" for p in parts), encoding="utf-8")
    # Re-encode for clean cuts / consistent params after zoompan segments
    run(
        [
            "ffmpeg",
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            str(lst),
            "-vf",
            f"fps={FPS},fade=t=in:st=0:d=0.2",
            *x264("4k"),
            str(out_mp4),
        ]
    )


def downscale_1080(src: Path, dst: Path) -> None:
    run(
        [
            "ffmpeg",
            "-y",
            "-i",
            str(src),
            "-vf",
            "scale=1920:1080:flags=lanczos",
            *x264("1080"),
            str(dst),
        ]
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path.home() / "Vídeos" / "termvox-demo",
    )
    parser.add_argument("--no-flash", action="store_true", help="Skip the 3.5s terminal flash")
    args = parser.parse_args()

    if not args.source.exists():
        print(f"source not found: {args.source}", file=sys.stderr)
        return 1
    if not shutil.which("ffmpeg"):
        print("ffmpeg required", file=sys.stderr)
        return 1

    src_dur = probe_duration(args.source)
    args.out_dir.mkdir(parents=True, exist_ok=True)
    master = args.out_dir / "termvox-linkedin-4k.mp4"
    linkedin = args.out_dir / "termvox-linkedin-1080p.mp4"

    print("style: midudev-like short product demo (jump cuts, no title cards)")
    print(f"source duration: {src_dur:.1f}s")

    with tempfile.TemporaryDirectory(prefix="termvox-midu-") as tmp:
        tmp_path = Path(tmp)
        parts: list[Path] = []

        if not args.no_flash:
            print("flash animation…")
            flash_frames = build_flash_animation()
            print(f"  {len(flash_frames)} frames (~{len(flash_frames)/FPS:.1f}s)")
            flash = tmp_path / "flash.mp4"
            encode_frames(flash_frames, flash)
            parts.append(flash)

        for i, (start, end, zoom, label) in enumerate(SEGMENTS):
            if start >= src_dur:
                continue
            end = min(end, src_dur - 0.05)
            if end <= start:
                continue
            out = tmp_path / f"seg_{i}_{label}.mp4"
            print(f"segment {label}: {start:.1f}-{end:.1f}s zoom={zoom}")
            render_segment(args.source, out, start, end, zoom)
            parts.append(out)

        if not parts:
            print("no segments produced", file=sys.stderr)
            return 1

        print("concat…")
        concat(parts, master)
        downscale_1080(master, linkedin)

    print()
    print("Wrote:")
    print(f"  {master}")
    print(f"  {linkedin}")
    print(f"  duration ≈ {probe_duration(linkedin):.1f}s")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
