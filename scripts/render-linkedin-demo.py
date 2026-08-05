#!/usr/bin/env python3
"""Render a LinkedIn-ready TermVox launch demo (4K master + 1080p).

Wraps a local screen recording with crisp terminal intro/outro cards,
lower-thirds, and letterboxing to 3840x2160.

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
BG = (11, 14, 20)
PANEL = (20, 26, 34)
TEAL = (46, 196, 182)
TEAL_SOFT = (61, 214, 198)
TEXT = (238, 243, 248)
MUTED = (140, 156, 172)
DIM = (70, 82, 96)
ACCENT_LINE = (42, 53, 68)

FONT_REG = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Regular.otf"
FONT_MED = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Medium.otf"
FONT_BOLD = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Bold.otf"
FONT_BLACK = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Black.otf"


def font(path: str, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(path, size=size)


def run(cmd: list[str]) -> None:
    print("+", " ".join(cmd[:8]), "..." if len(cmd) > 8 else "", flush=True)
    subprocess.run(cmd, check=True)


def new_canvas() -> Image.Image:
    img = Image.new("RGB", (W, H), BG)
    draw = ImageDraw.Draw(img)
    # subtle top grid lines
    for y in range(0, H, 120):
        draw.line([(0, y), (W, y)], fill=(16, 20, 28), width=1)
    for x in range(0, W, 160):
        draw.line([(x, 0), (x, H)], fill=(16, 20, 28), width=1)
    # left teal rail
    draw.rectangle([0, 0, 18, H], fill=TEAL)
    return img


def draw_badge(draw: ImageDraw.ImageDraw, label: str, x: int, y: int) -> None:
    f = font(FONT_MED, 42)
    pad_x, pad_y = 28, 14
    bbox = draw.textbbox((0, 0), label, font=f)
    tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
    draw.rounded_rectangle(
        [x, y, x + tw + pad_x * 2, y + th + pad_y * 2],
        radius=18,
        fill=(22, 40, 42),
        outline=TEAL,
        width=3,
    )
    draw.text((x + pad_x, y + pad_y - 4), label, font=f, fill=TEAL_SOFT)


def draw_waveform(draw: ImageDraw.ImageDraw, cx: int, cy: int, t: float, amp: float = 1.0) -> None:
    bars = 28
    gap = 18
    total_w = bars * gap
    x0 = cx - total_w // 2
    for i in range(bars):
        phase = t * 6.0 + i * 0.55
        h = int((22 + 70 * abs(math.sin(phase)) * amp) * (0.55 + 0.45 * abs(math.sin(i * 0.4))))
        x = x0 + i * gap
        color = TEAL if i % 3 else TEAL_SOFT
        draw.rounded_rectangle([x, cy - h, x + 8, cy + h], radius=4, fill=color)


def save_frame(img: Image.Image, path: Path) -> None:
    img.save(path, "PNG", optimize=False)


def render_intro(frames_dir: Path, logo: Path | None, seconds: float = 6.5) -> int:
    frames_dir.mkdir(parents=True, exist_ok=True)
    n = int(seconds * FPS)
    logo_img = None
    if logo and logo.exists():
        logo_img = Image.open(logo).convert("RGBA").resize((220, 220), Image.Resampling.LANCZOS)

    title_f = font(FONT_BLACK, 160)
    sub_f = font(FONT_MED, 64)
    cmd_f = font(FONT_BOLD, 56)
    hint_f = font(FONT_REG, 44)

    line1 = "npm install -g termvox"
    line2 = "termvox shell --agent opencode"

    for i in range(n):
        t = i / FPS
        img = new_canvas()
        draw = ImageDraw.Draw(img)

        if logo_img:
            img.paste(logo_img, (120, 140), logo_img)

        draw.text((400, 170), "TERMVOX", font=title_f, fill=TEXT)
        draw_badge(draw, "alpha preview", 400, 360)
        draw.text(
            (400, 470),
            "Voz local para coding agents en la terminal",
            font=sub_f,
            fill=MUTED,
        )

        # terminal panel
        px, py, pw, ph = 360, 700, W - 720, 900
        draw.rounded_rectangle([px, py, px + pw, py + ph], radius=28, fill=PANEL, outline=ACCENT_LINE, width=3)
        draw.ellipse([px + 48, py + 40, px + 78, py + 70], fill=(255, 95, 87))
        draw.ellipse([px + 98, py + 40, px + 128, py + 70], fill=(255, 189, 46))
        draw.ellipse([px + 148, py + 40, px + 178, py + 70], fill=(39, 201, 63))
        draw.text((px + 230, py + 34), "~/proyecto — termvox shell", font=hint_f, fill=DIM)

        # typing
        typed1 = min(len(line1), max(0, int((t - 1.0) * 18)))
        typed2 = min(len(line2), max(0, int((t - 3.2) * 16)))
        y1 = py + 180
        draw.text((px + 80, y1), "$ ", font=cmd_f, fill=TEAL)
        chunk1 = line1[:typed1]
        draw.text((px + 140, y1), chunk1, font=cmd_f, fill=TEXT)
        cursor_x1 = px + 140 + int(draw.textlength(chunk1, font=cmd_f))
        if 1.0 < t < 3.0 and typed1 < len(line1) and int(t * 4) % 2 == 0:
            draw.rectangle([cursor_x1, y1 + 8, cursor_x1 + 28, y1 + 58], fill=TEAL)

        if t >= 3.0:
            draw.text((px + 80, y1 + 110), "✓ installed", font=hint_f, fill=TEAL_SOFT)

        y2 = y1 + 220
        draw.text((px + 80, y2), "$ ", font=cmd_f, fill=TEAL)
        chunk2 = line2[:typed2]
        draw.text((px + 140, y2), chunk2, font=cmd_f, fill=TEXT)
        cursor_x2 = px + 140 + int(draw.textlength(chunk2, font=cmd_f))
        if t > 3.2 and typed2 < len(line2) and int(t * 4) % 2 == 0:
            draw.rectangle([cursor_x2, y2 + 8, cursor_x2 + 28, y2 + 58], fill=TEAL)

        if t >= 5.0:
            draw.text(
                (px + 80, y2 + 130),
                "F8 / Ctrl+Space  ·  Whisper on-device  ·  confirm before send",
                font=hint_f,
                fill=MUTED,
            )
            draw_waveform(draw, px + pw // 2, py + ph - 140, t, amp=1.0)

        save_frame(img, frames_dir / f"frame_{i:05d}.png")
    return n


def render_outro(frames_dir: Path, logo: Path | None, seconds: float = 5.5) -> int:
    frames_dir.mkdir(parents=True, exist_ok=True)
    n = int(seconds * FPS)
    logo_img = None
    if logo and logo.exists():
        logo_img = Image.open(logo).convert("RGBA").resize((180, 180), Image.Resampling.LANCZOS)

    title_f = font(FONT_BLACK, 120)
    body_f = font(FONT_MED, 58)
    cmd_f = font(FONT_BOLD, 64)
    small_f = font(FONT_REG, 44)

    for i in range(n):
        t = i / FPS
        img = new_canvas()
        draw = ImageDraw.Draw(img)
        if logo_img:
            img.paste(logo_img, (160, 200), logo_img)
        draw.text((400, 230), "Pruébalo", font=title_f, fill=TEXT)
        draw.text((400, 400), "Alpha abierta · feedback welcome", font=body_f, fill=MUTED)

        draw.rounded_rectangle([360, 560, W - 360, 980], radius=28, fill=PANEL, outline=TEAL, width=3)
        draw.text((460, 640), "$ npm install -g termvox", font=cmd_f, fill=TEXT)
        draw.text((460, 760), "$ termvox shell --agent opencode", font=cmd_f, fill=TEAL_SOFT)

        draw.text((400, 1120), "github.com/Jeronimo0228/termvox", font=body_f, fill=TEXT)
        draw.text((400, 1220), "npmjs.com/package/termvox", font=body_f, fill=MUTED)
        fade = min(1.0, max(0.0, (t - 0.3) / 0.8))
        draw_waveform(draw, W // 2, 1700, t, amp=0.4 + 0.6 * fade)
        draw.text(
            (W // 2 - 520, 1900),
            "Whisper local · Cursor · OpenCode · Claude · Codex",
            font=small_f,
            fill=DIM,
        )
        save_frame(img, frames_dir / f"frame_{i:05d}.png")
    return n


def x264_args(kind: str) -> list[str]:
    # Force enough bitrate so LinkedIn's re-encode keeps terminal text readable.
    if kind == "4k":
        return [
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-profile:v",
            "high",
            "-preset",
            "medium",
            "-b:v",
            "28M",
            "-maxrate",
            "36M",
            "-bufsize",
            "56M",
            "-x264-params",
            "aq-mode=3:aq-strength=1.0",
            "-r",
            str(FPS),
            "-movflags",
            "+faststart",
        ]
    return [
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-profile:v",
        "high",
        "-preset",
        "medium",
        "-b:v",
        "12M",
        "-maxrate",
        "16M",
        "-bufsize",
        "24M",
        "-x264-params",
        "aq-mode=3:aq-strength=1.0",
        "-r",
        str(FPS),
        "-movflags",
        "+faststart",
    ]


def encode_frames(frames_dir: Path, out_mp4: Path, n_frames: int) -> None:
    run(
        [
            "ffmpeg",
            "-y",
            "-framerate",
            str(FPS),
            "-i",
            str(frames_dir / "frame_%05d.png"),
            "-frames:v",
            str(n_frames),
            *x264_args("4k"),
            str(out_mp4),
        ]
    )


def ass_time(seconds: float) -> str:
    seconds = max(0.0, seconds)
    h = int(seconds // 3600)
    m = int((seconds % 3600) // 60)
    s = seconds % 60
    return f"{h}:{m:02d}:{s:05.2f}"


def write_ass(path: Path, duration: float) -> None:
    # ASS PlayRes matches 4K canvas; events timed to typical shell demo pacing.
    end = ass_time(max(1.0, duration - 0.05))
    content = f"""[Script Info]
Title: TermVox LinkedIn
ScriptType: v4.00+
PlayResX: {W}
PlayResY: {H}
WrapStyle: 0

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Lower,Source Code Pro,{72},&H00E6F3F8,&H000000FF,&H00140E0B,&H99000000,-1,0,0,0,100,100,0,0,1,4,0,2,80,80,120,1
Style: Tag,Source Code Pro,{54},&H00B6C42E,&H000000FF,&H00140E0B,&HAA000000,-1,0,0,0,100,100,0,0,1,3,0,7,80,80,80,1
Style: Hook,Source Code Pro,{64},&H00E6F3F8,&H000000FF,&H00140E0B,&HAA000000,-1,0,0,0,100,100,0,0,1,4,0,8,80,80,160,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.20,0:00:04.50,Tag,,0,0,0,,TERMVOX  ·  shell + OpenCode
Dialogue: 0,0:00:00.40,0:00:05.00,Hook,,0,0,0,,Hablas → Whisper local → el agente recibe el prompt
Dialogue: 0,0:00:05.00,0:00:12.00,Lower,,0,0,0,,Mic bar integrada  ·  F8 / Ctrl+Space
Dialogue: 0,0:00:12.00,0:00:22.00,Lower,,0,0,0,,Prompt por voz (ES)  ·  confirmación antes de enviar
Dialogue: 0,0:00:22.00,0:00:38.00,Lower,,0,0,0,,OpenCode trabaja en el mismo TUI
Dialogue: 0,0:00:38.00,{end},Lower,,0,0,0,,Alpha preview  ·  github.com/Jeronimo0228/termvox
"""
    path.write_text(content, encoding="utf-8")


def process_source(source: Path, ass: Path, out_mp4: Path) -> None:
    # Pad to 4K 16:9, upscale with lanczos, light sharpen, burn ASS, force 30fps.
    pad_color = f"0x{BG[0]:02x}{BG[1]:02x}{BG[2]:02x}"
    vf = (
        f"fps={FPS},"
        f"scale={W}:{H}:force_original_aspect_ratio=decrease:flags=lanczos,"
        f"pad={W}:{H}:(ow-iw)/2:(oh-ih)/2:{pad_color},"
        "unsharp=5:5:0.6:5:5:0.0,"
        f"ass={ass.as_posix()}"
    )
    run(
        [
            "ffmpeg",
            "-y",
            "-i",
            str(source),
            "-vf",
            vf,
            "-an",
            *x264_args("4k"),
            str(out_mp4),
        ]
    )


def concat(parts: list[Path], out_mp4: Path) -> None:
    lst = out_mp4.with_suffix(".txt")
    lst.write_text("".join(f"file '{p.resolve()}'\n" for p in parts), encoding="utf-8")
    # Stream-copy: segments already share codec/params.
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
            *x264_args("1080"),
            str(dst),
        ]
    )


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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path.home() / "Vídeos" / "termvox-demo",
    )
    parser.add_argument(
        "--logo",
        type=Path,
        default=Path.home() / "Vídeos" / "termvox-demo" / "logo-512.png",
    )
    args = parser.parse_args()

    if not args.source.exists():
        print(f"source not found: {args.source}", file=sys.stderr)
        return 1
    if not shutil.which("ffmpeg"):
        print("ffmpeg required", file=sys.stderr)
        return 1

    args.out_dir.mkdir(parents=True, exist_ok=True)
    duration = probe_duration(args.source)
    print(f"source duration: {duration:.2f}s")

    with tempfile.TemporaryDirectory(prefix="termvox-demo-") as tmp:
        tmp_path = Path(tmp)
        intro_frames = tmp_path / "intro"
        outro_frames = tmp_path / "outro"
        n_intro = render_intro(intro_frames, args.logo)
        n_outro = render_outro(outro_frames, args.logo)
        intro_mp4 = tmp_path / "intro.mp4"
        outro_mp4 = tmp_path / "outro.mp4"
        middle_mp4 = tmp_path / "middle.mp4"
        encode_frames(intro_frames, intro_mp4, n_intro)
        encode_frames(outro_frames, outro_mp4, n_outro)

        ass = tmp_path / "captions.ass"
        write_ass(ass, duration)
        process_source(args.source, ass, middle_mp4)

        master = args.out_dir / "termvox-linkedin-4k.mp4"
        linkedin_1080 = args.out_dir / "termvox-linkedin-1080p.mp4"
        concat([intro_mp4, middle_mp4, outro_mp4], master)
        downscale_1080(master, linkedin_1080)

        # also keep a silent vertical-safe square crop tip sheet? skip — LinkedIn landscape is fine.

    print()
    print("Wrote:")
    print(f"  {master}")
    print(f"  {linkedin_1080}")
    print("Tip: LinkedIn compresses uploads — prefer the 1080p file for smoother delivery;")
    print("keep the 4K master as archive / YouTube.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
