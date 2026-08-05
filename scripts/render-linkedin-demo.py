#!/usr/bin/env python3
"""Practical TermVox pitch demo: terminal setup animation + cleaned session.

- Animated terminal walkthrough (config / doctor / models / shell)
- Real screen recording, full-bleed 16:9 (no letterbox bars)
- ANSI glitch fragments covered with drawbox
- No marketing title cards or burned-in captions

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
FONT_BOLD = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Bold.otf"

# Source recording is 1920x924. Cover center ANSI garbage + corrupted SGR crumbs
# in the composer strip. Times are source-absolute; process_recording shifts by --start.
GLITCH_BOXES_SRC = [
    # Center fragment like "38;2;255;255;255m"
    (560, 430, 620, 90, "0x0A0C10", 10.5, 34.5),
    # Corrupted composer crumbs e.g. "8;2;30;30;"
    (220, 700, 900, 50, "0x141820", 10.5, 40.0),
]


def font(path: str, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(path, size=size)


def run(cmd: list[str]) -> None:
    print("+", " ".join(str(c) for c in cmd[:14]), "..." if len(cmd) > 14 else "", flush=True)
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
    """Fullscreen terminal renderer with typing + scrolling lines."""

    def __init__(self) -> None:
        self.lines: list[tuple[str, tuple[int, int, int]]] = []
        self.prompt_cwd = "~/proyecto"
        self.partial = ""
        self.partial_color = FG
        self.show_cursor = True
        self.f_reg = font(FONT_REG, 44)
        self.f_med = font(FONT_MED, 44)
        self.f_bold = font(FONT_BOLD, 44)
        self.pad_x = 96
        self.pad_y = 72
        self.line_h = 58
        self.max_visible = 28

    def add(self, text: str, color: tuple[int, int, int] = FG) -> None:
        self.lines.append((text, color))

    def clear(self) -> None:
        self.lines.clear()
        self.partial = ""

    def render(self) -> Image.Image:
        img = Image.new("RGB", (W, H), BG)
        draw = ImageDraw.Draw(img)
        # subtle top bar like a real VTE
        draw.rectangle([0, 0, W, 48], fill=PANEL)
        draw.ellipse([28, 14, 48, 34], fill=(255, 95, 87))
        draw.ellipse([60, 14, 80, 34], fill=(255, 189, 46))
        draw.ellipse([92, 14, 112, 34], fill=(39, 201, 63))
        draw.text((140, 10), f"termvox — {self.prompt_cwd}", font=font(FONT_MED, 28), fill=DIM)

        visible = self.lines[-self.max_visible :]
        y = self.pad_y + 40
        for text, color in visible:
            draw.text((self.pad_x, y), text, font=self.f_reg, fill=color)
            y += self.line_h

        # active input line
        prompt = f"{self.prompt_cwd} ❯ "
        draw.text((self.pad_x, y), prompt, font=self.f_med, fill=TEAL)
        px = self.pad_x + int(draw.textlength(prompt, font=self.f_med))
        draw.text((px, y), self.partial, font=self.f_reg, fill=self.partial_color)
        if self.show_cursor:
            cx = px + int(draw.textlength(self.partial, font=self.f_reg))
            draw.rectangle([cx + 4, y + 8, cx + 28, y + 48], fill=CURSOR)
        return img


def type_command(scene: TerminalScene, frames: list[Image.Image], cmd: str, cps: float = 34.0) -> None:
    scene.partial = ""
    scene.show_cursor = True
    for _ in range(4):
        frames.append(scene.render())
    n = max(1, int(len(cmd) / cps * FPS))
    for i in range(n + 1):
        chars = int(len(cmd) * i / max(1, n))
        scene.partial = cmd[:chars]
        scene.show_cursor = (i % 2 == 0) or chars < len(cmd)
        frames.append(scene.render())
    for _ in range(3):
        scene.show_cursor = True
        frames.append(scene.render())
    scene.add(f"{scene.prompt_cwd} ❯ {cmd}", FG)
    scene.partial = ""
    scene.show_cursor = False
    frames.append(scene.render())


def emit_lines(
    scene: TerminalScene,
    frames: list[Image.Image],
    lines: list[tuple[str, tuple[int, int, int]]],
    hold: int = 2,
) -> None:
    for text, color in lines:
        scene.add(text, color)
        for _ in range(hold):
            frames.append(scene.render())


def hold(scene: TerminalScene, frames: list[Image.Image], frames_n: int) -> None:
    scene.show_cursor = True
    for i in range(frames_n):
        scene.show_cursor = (i // 8) % 2 == 0
        frames.append(scene.render())


def build_setup_animation() -> list[Image.Image]:
    scene = TerminalScene()
    frames: list[Image.Image] = []

    hold(scene, frames, 6)
    type_command(scene, frames, "cd ~/proyecto")
    emit_lines(scene, frames, [("", DIM)], hold=1)

    type_command(scene, frames, "termvox init --preset opencode --force")
    emit_lines(
        scene,
        frames,
        [
            ("wrote termvox.toml", TEAL),
            ("preset=opencode  display=shell  language=es", DIM),
            ("", DIM),
        ],
        hold=2,
    )

    type_command(scene, frames, "termvox config show")
    emit_lines(
        scene,
        frames,
        [
            ('performance_profile = "balanced"', FG),
            ('speech_engine = "whisper"', FG),
            ('agent = "opencode"', FG),
            ('language = "es"', FG),
            ("confirmation = true", FG),
            ("", DIM),
            ("[whisper]", DIM),
            ('model = "~/.local/share/termvox/models/ggml-base.bin"', FG),
            ("", DIM),
            ("[agents.opencode]", DIM),
            ('display = "shell"', FG),
            ("", DIM),
        ],
        hold=1,
    )
    hold(scene, frames, 10)

    type_command(scene, frames, "termvox models install accurate")
    for pct in (12, 34, 58, 79, 100):
        bar = "█" * (pct // 5) + "░" * (20 - pct // 5)
        scene.add(f"downloading whisper-base  [{bar}] {pct}%", DIM)
        for _ in range(3):
            frames.append(scene.render())
    emit_lines(
        scene,
        frames,
        [
            ("verified sha256", GREEN),
            ("installed ggml-base.bin (~142 MiB)", TEAL),
            ("", DIM),
        ],
        hold=2,
    )

    type_command(scene, frames, "termvox doctor")
    emit_lines(
        scene,
        frames,
        [
            ("TermVox doctor", FG),
            ("", DIM),
            ("[ok] configuration      valid", GREEN),
            ("[ok] microphone         ready (PipeWire)", GREEN),
            ("[ok] speech/whisper     ready (ggml-base.bin)", GREEN),
            ("[ok] agent/opencode     1.18.13 (credentials ok)", GREEN),
            ("[ok] agent/cursor       credentials present", GREEN),
            ("", DIM),
            ("Selected agent: opencode", FG),
            ("  performance_profile = balanced", DIM),
            ("  display = shell", DIM),
            ("  hint: use termvox shell — mic bar inside the agent TUI", TEAL),
            ("", DIM),
        ],
        hold=2,
    )
    hold(scene, frames, 14)

    type_command(scene, frames, "termvox shell --agent opencode --fresh")
    emit_lines(
        scene,
        frames,
        [
            ("launching opencode in PTY…", DIM),
            ("whisper prewarm deferred until first voice toggle", DIM),
            ("mic bar: F8 / Ctrl+Space   exit: Ctrl+\\", TEAL),
            ("", DIM),
        ],
        hold=3,
    )
    for i in range(12):
        img = scene.render()
        overlay = Image.new("RGB", (W, H), (0, 0, 0))
        frames.append(Image.blend(img, overlay, i / 11))

    return frames


def encode_frames(frames: list[Image.Image], out_mp4: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="termvox-anim-") as tmp:
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


def process_recording(source: Path, out_mp4: Path, start: float) -> None:
    duration = probe_duration(source)
    usable = max(0.5, duration - start)
    # Full-bleed: scale to COVER 16:9 then crop center (no black bars).
    # Hide ANSI glitch boxes in source pixel space before scale.
    # With -ss before -i, filter timestamps restart at 0.
    boxes = []
    for x, y, w, h, color, t0, t1 in GLITCH_BOXES_SRC:
        a = max(0.0, t0 - start)
        b = max(0.0, t1 - start)
        boxes.append(
            f"drawbox=x={x}:y={y}:w={w}:h={h}:color={color}@1:t=fill:enable='between(t,{a:.2f},{b:.2f})'"
        )
    box_f = ",".join(boxes) + "," if boxes else ""
    # Fit full width (no side crop — terminal text stays intact). Pad top/bottom
    # with the same near-black as the TUI so it reads as full-screen.
    vf = (
        f"{box_f}"
        f"fps={FPS},"
        f"scale={W}:{H}:force_original_aspect_ratio=decrease:flags=lanczos,"
        f"pad={W}:{H}:(ow-iw)/2:(oh-ih)/2:0x0A0C10,"
        "eq=contrast=1.05:brightness=0.015:saturation=1.04,"
        "unsharp=5:5:0.65:5:5:0.0,"
        "fade=t=in:st=0:d=0.25,"
        f"fade=t=out:st={max(0.0, usable - 0.45):.3f}:d=0.45"
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
            f"{usable:.3f}",
            "-vf",
            vf,
            "-an",
            *x264("4k"),
            str(out_mp4),
        ]
    )


def concat(parts: list[Path], out_mp4: Path) -> None:
    lst = out_mp4.with_suffix(".txt")
    lst.write_text("".join(f"file '{p.resolve()}'\n" for p in parts), encoding="utf-8")
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
            "-c",
            "copy",
            "-movflags",
            "+faststart",
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
    parser.add_argument("--start", type=float, default=0.8)
    parser.add_argument("--skip-animation", action="store_true")
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

    with tempfile.TemporaryDirectory(prefix="termvox-demo-") as tmp:
        tmp_path = Path(tmp)
        session = tmp_path / "session.mp4"
        print("processing session (full-bleed + hide ANSI glitch)…")
        process_recording(args.source, session, args.start)

        parts = [session]
        if not args.skip_animation:
            print("rendering terminal setup animation…")
            anim_frames = build_setup_animation()
            print(f"animation frames: {len(anim_frames)} (~{len(anim_frames)/FPS:.1f}s)")
            anim = tmp_path / "setup.mp4"
            encode_frames(anim_frames, anim)
            parts = [anim, session]

        print("concat + export…")
        concat(parts, master)
        downscale_1080(master, linkedin)

    print()
    print("Wrote:")
    print(f"  {master}")
    print(f"  {linkedin}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
