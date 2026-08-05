#!/usr/bin/env python3
"""Polished TermVox pitch demo — smooth terminal → live session.

Avoids the empty OpenCode splash and zoom-crop artifacts:
  1) Clean terminal sequence (doctor / shell / listening / transcript)
  2) Crossfade into the dense agent-working portion of the recording
  3) Crop away the right sidebar so the conversation fills the frame
  4) Cover ANSI glitch crumbs; no zoompan

Usage:
  /usr/bin/python3.14 scripts/render-linkedin-demo.py \\
    --source "/path/to/recording.mp4" \\
    --out-dir "$HOME/Vídeos/termvox-demo"
"""

from __future__ import annotations

import argparse
import math
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

W, H = 3840, 2160
FPS = 30
BG = (8, 10, 14)
FG = (232, 238, 244)
DIM = (118, 130, 144)
TEAL = (42, 210, 190)
TEAL_HOT = (120, 255, 230)
GREEN = (80, 220, 140)
PANEL = (16, 20, 26)
BAR = (22, 48, 52)
CURSOR = (42, 210, 190)

FONT_REG = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Regular.otf"
FONT_MED = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Medium.otf"
FONT_BOLD = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Bold.otf"

# Only use the dense agent-working part of the 2026-08-04 23-39-46 take.
# Skip empty OpenCode splash (≈11–28s) and the dead command prompt lead-in.
LIVE_START = 36.0
LIVE_END = 57.2

# Source is 1920×924. Crop out the right sidebar (~25%) so chat fills the frame.
CROP_W = 1420
CROP_X = 0
CROP_Y = 0
CROP_H = 924

# ANSI crumbs on source coords (before crop). Cover generously.
GLITCH_BOXES = [
    # center garbage "38;2;255;255;255m"
    "drawbox=x=520:y=380:w=700:h=120:color=0x080A0E@1:t=fill",
    # composer crumbs "8;2;30;30;"
    "drawbox=x=180:y=680:w=980:h=70:color=0x101418@1:t=fill",
]


def font(path: str, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(path, size=size)


def run(cmd: list[str]) -> None:
    print("+", " ".join(str(c) for c in cmd[:18]), "..." if len(cmd) > 18 else "", flush=True)
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
        rate = ["-b:v", "26M", "-maxrate", "34M", "-bufsize", "52M"]
    else:
        rate = ["-b:v", "11M", "-maxrate", "15M", "-bufsize", "22M"]
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


def draw_waveform(draw: ImageDraw.ImageDraw, cx: int, cy: int, t: float, amp: float = 1.0) -> None:
    bars, gap, bw = 36, 18, 10
    x0 = cx - (bars * gap) // 2
    for i in range(bars):
        phase = t * 8.5 + i * 0.45
        h = int((18 + 90 * abs(math.sin(phase)) * amp) * (0.5 + 0.5 * abs(math.sin(i * 0.35))))
        x = x0 + i * gap
        col = TEAL_HOT if i % 4 == 0 else TEAL
        draw.rounded_rectangle([x, cy - h, x + bw, cy + h], radius=4, fill=col)


class Terminal:
    def __init__(self) -> None:
        self.lines: list[tuple[str, tuple[int, int, int]]] = []
        self.cwd = "~/proyecto"
        self.partial = ""
        self.cursor = True
        self.mic_listening = False
        self.mic_t = 0.0
        self.f = font(FONT_REG, 50)
        self.fm = font(FONT_MED, 50)
        self.fb = font(FONT_BOLD, 50)
        self.pad_x = 120
        self.line_h = 66

    def add(self, text: str, color: tuple[int, int, int] = FG) -> None:
        self.lines.append((text, color))

    def render(self) -> Image.Image:
        img = Image.new("RGB", (W, H), BG)
        d = ImageDraw.Draw(img)
        # chrome
        d.rectangle([0, 0, W, 56], fill=PANEL)
        for i, col in enumerate(((255, 95, 87), (255, 189, 46), (39, 201, 63))):
            x = 32 + i * 36
            d.ellipse([x, 18, x + 22, 40], fill=col)
        d.text((150, 14), f"termvox — {self.cwd}", font=font(FONT_MED, 32), fill=DIM)

        y = 110
        for text, color in self.lines[-22:]:
            d.text((self.pad_x, y), text, font=self.f, fill=color)
            y += self.line_h

        prompt = f"{self.cwd} ❯ "
        d.text((self.pad_x, y), prompt, font=self.fm, fill=TEAL)
        px = self.pad_x + int(d.textlength(prompt, font=self.fm))
        d.text((px, y), self.partial, font=self.f, fill=FG)
        if self.cursor and not self.mic_listening:
            cx = px + int(d.textlength(self.partial, font=self.f))
            d.rectangle([cx + 6, y + 12, cx + 32, y + 54], fill=CURSOR)

        # TermVox mic bar (product moment)
        if self.mic_listening:
            by = H - 120
            d.rounded_rectangle([80, by, W - 80, by + 88], radius=18, fill=BAR, outline=TEAL, width=3)
            d.text((120, by + 22), "● Escuchando", font=font(FONT_BOLD, 40), fill=TEAL_HOT)
            d.text((520, by + 28), "F8 / Ctrl+Space", font=font(FONT_MED, 34), fill=DIM)
            draw_waveform(d, W // 2 + 400, by + 44, self.mic_t, amp=1.15)
        else:
            # idle status strip like real shell
            d.rectangle([0, H - 52, W, H], fill=(28, 32, 38))
            d.text(
                (40, H - 40),
                "TermVox  ·  OpenCode  ·  ES  ·  listo  ·  voz F8/Ctrl+Space  ·  salir Ctrl+\\",
                font=font(FONT_MED, 28),
                fill=DIM,
            )
        return img


def type_cmd(term: Terminal, frames: list[Image.Image], cmd: str, cps: float = 28.0) -> None:
    term.partial = ""
    term.cursor = True
    for _ in range(4):
        frames.append(term.render())
    n = max(8, int(len(cmd) / cps * FPS))
    for i in range(n + 1):
        term.partial = cmd[: int(len(cmd) * i / max(1, n))]
        term.cursor = i % 2 == 0 or i < n
        frames.append(term.render())
    for _ in range(3):
        frames.append(term.render())
    term.add(f"{term.cwd} ❯ {cmd}", FG)
    term.partial = ""
    term.cursor = False
    frames.append(term.render())


def emit(term: Terminal, frames: list[Image.Image], lines: list[tuple[str, tuple[int, int, int]]], hold: int = 2) -> None:
    for text, color in lines:
        term.add(text, color)
        for _ in range(hold):
            frames.append(term.render())


def hold(term: Terminal, frames: list[Image.Image], n: int) -> None:
    for i in range(n):
        term.cursor = (i // 10) % 2 == 0
        frames.append(term.render())


def build_terminal() -> list[Image.Image]:
    """Aesthetic terminal story that leads into the live agent beat."""
    term = Terminal()
    frames: list[Image.Image] = []

    hold(term, frames, 8)
    type_cmd(term, frames, "termvox doctor")
    emit(
        term,
        frames,
        [
            ("", DIM),
            ("TermVox doctor", FG),
            ("[ok] configuration   valid", GREEN),
            ("[ok] microphone      ready", GREEN),
            ("[ok] speech/whisper  ggml-base", GREEN),
            ("[ok] agent/opencode  authenticated", GREEN),
            ("", DIM),
        ],
        hold=2,
    )
    hold(term, frames, 10)

    type_cmd(term, frames, "termvox shell --agent opencode --fresh")
    emit(
        term,
        frames,
        [
            ("launching opencode…", DIM),
            ("mic bar attached  ·  F8 / Ctrl+Space", TEAL),
            ("", DIM),
        ],
        hold=2,
    )

    # Voice moment — the product
    term.mic_listening = True
    for i in range(int(2.2 * FPS)):
        term.mic_t = i / FPS
        frames.append(term.render())

    term.mic_listening = False
    emit(term, frames, [("", DIM), ("▸ transcript", TEAL)], hold=2)

    line1 = "Necesito una arquitectura para agentes de IA"
    line2 = "que automaticen trabajos en la agencia de viajes."
    term.add("", FG)
    idx = len(term.lines) - 1
    for i in range(len(line1) + 1):
        term.lines[idx] = (line1[:i], FG)
        if i % 2 == 0:
            frames.append(term.render())
    term.add("", FG)
    idx2 = len(term.lines) - 1
    for i in range(len(line2) + 1):
        term.lines[idx2] = (line2[:i], FG)
        if i % 2 == 0:
            frames.append(term.render())
    hold(term, frames, 14)

    emit(
        term,
        frames,
        [
            ("", DIM),
            ("confirm → send to OpenCode", TEAL),
            ("", DIM),
        ],
        hold=3,
    )
    last = term.render()
    for i in range(10):
        frames.append(Image.blend(last, Image.new("RGB", (W, H), (0, 0, 0)), i / 9))
    return frames


def encode_frames(frames: list[Image.Image], out_mp4: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="termvox-term-") as tmp:
        tmp_path = Path(tmp)
        for i, fr in enumerate(frames):
            fr.save(tmp_path / f"frame_{i:05d}.png")
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


def xfade_concat(a: Path, b: Path, out: Path, fade: float = 0.55) -> None:
    """Smooth crossfade empalme between terminal and live session."""
    da = probe_duration(a)
    offset = max(0.05, da - fade)
    run(
        [
            "ffmpeg",
            "-y",
            "-i",
            str(a),
            "-i",
            str(b),
            "-filter_complex",
            f"[0:v][1:v]xfade=transition=fade:duration={fade}:offset={offset:.3f},format=yuv420p[v]",
            "-map",
            "[v]",
            *x264("4k"),
            str(out),
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
    args = parser.parse_args()

    if not args.source.exists():
        print(f"source not found: {args.source}", file=sys.stderr)
        return 1
    if not shutil.which("ffmpeg"):
        print("ffmpeg required", file=sys.stderr)
        return 1

    src_dur = probe_duration(args.source)
    if LIVE_END > src_dur:
        print(f"warning: LIVE_END={LIVE_END} > source {src_dur:.1f}; clamping", file=sys.stderr)

    args.out_dir.mkdir(parents=True, exist_ok=True)
    master = args.out_dir / "termvox-linkedin-4k.mp4"
    linkedin = args.out_dir / "termvox-linkedin-1080p.mp4"

    print("rebuild: smooth terminal → live agent (no splash, no zoom crop)")
    with tempfile.TemporaryDirectory(prefix="termvox-smooth-") as tmp:
        tmp_path = Path(tmp)
        print("terminal sequence…")
        frames = build_terminal()
        print(f"  {len(frames)} frames (~{len(frames)/FPS:.1f}s)")
        term_mp4 = tmp_path / "terminal.mp4"
        encode_frames(frames, term_mp4)

        end = min(LIVE_END, src_dur - 0.05)
        print(f"live session {LIVE_START:.1f}–{end:.1f}s (sidebar cropped)…")
        live_mp4 = tmp_path / "live.mp4"
        dur = end - LIVE_START
        boxes = ",".join(GLITCH_BOXES)
        vf = (
            f"{boxes},"
            f"crop={CROP_W}:{CROP_H}:{CROP_X}:{CROP_Y},"
            f"fps={FPS},"
            f"scale={W}:{H}:flags=lanczos,"
            "eq=contrast=1.07:brightness=0.025:saturation=1.06,"
            "unsharp=5:5:0.75:5:5:0.0,"
            f"fade=t=in:st=0:d=0.45,"
            f"fade=t=out:st={max(0.2, dur - 0.5):.2f}:d=0.5"
        )
        run(
            [
                "ffmpeg",
                "-y",
                "-ss",
                f"{LIVE_START:.3f}",
                "-i",
                str(args.source),
                "-t",
                f"{dur:.3f}",
                "-vf",
                vf,
                "-an",
                *x264("4k"),
                str(live_mp4),
            ]
        )

        print("xfade empalme…")
        xfade_concat(term_mp4, live_mp4, master, fade=0.6)
        downscale_1080(master, linkedin)

    print()
    print("Wrote:")
    print(f"  {master}")
    print(f"  {linkedin}")
    print(f"  duration ≈ {probe_duration(linkedin):.1f}s")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
