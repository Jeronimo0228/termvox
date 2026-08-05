#!/usr/bin/env python3
"""Render a scroll-stopping LinkedIn TermVox demo (4K master + 1080p).

Cold-open hook cards + kinetic captions wrapped around a screen recording.

Usage:
  /usr/bin/python3.14 scripts/render-linkedin-demo.py \\
    --source "/path/to/recording.mp4" \\
    --out-dir "$HOME/Vídeos/termvox-demo"
"""

from __future__ import annotations

import argparse
import math
import random
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

W, H = 3840, 2160
FPS = 30
_NOISE_TILE: Image.Image | None = None

# Broadcast / signal-room palette (not soft SaaS purple)
BG = (6, 8, 12)
INK = (232, 240, 246)
MUTED = (122, 138, 154)
DIM = (58, 70, 84)
TEAL = (0, 232, 198)
TEAL_HOT = (120, 255, 230)
PANEL = (12, 16, 22)
DANGER = (255, 72, 88)
REC_RED = (255, 48, 64)

FONT_DISPLAY = "/usr/share/fonts/librefranklin/LibreFranklin-Black.otf"
FONT_DISPLAY_XB = "/usr/share/fonts/librefranklin/LibreFranklin-ExtraBold.otf"
FONT_NARROW = "/usr/share/fonts/liberation-narrow/LiberationSansNarrow-Bold.ttf"
FONT_MONO = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Bold.otf"
FONT_MONO_REG = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Regular.otf"
FONT_MONO_BLACK = "/usr/share/fonts/adobe-source-code-pro-fonts/SourceCodePro-Black.otf"


def font(path: str, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(path, size=size)


def run(cmd: list[str]) -> None:
    print("+", " ".join(str(c) for c in cmd[:10]), "..." if len(cmd) > 10 else "", flush=True)
    subprocess.run(cmd, check=True)


def lerp(a: float, b: float, t: float) -> float:
    return a + (b - a) * max(0.0, min(1.0, t))


def ease_out_cubic(t: float) -> float:
    t = max(0.0, min(1.0, t))
    return 1 - (1 - t) ** 3


def ease_out_back(t: float) -> float:
    t = max(0.0, min(1.0, t))
    c = 1.70158
    return 1 + (c + 1) * (t - 1) ** 3 + c * (t - 1) ** 2


def _noise_tile() -> Image.Image:
    global _NOISE_TILE
    if _NOISE_TILE is None:
        rng = random.Random(42)
        tile = Image.new("RGB", (256, 256), (128, 128, 128))
        px = tile.load()
        for y in range(256):
            for x in range(256):
                v = 128 + rng.randint(-18, 18)
                px[x, y] = (v, v, v)
        _NOISE_TILE = tile.filter(ImageFilter.GaussianBlur(0.5))
    return _NOISE_TILE


def add_grain(img: Image.Image) -> Image.Image:
    """Light grain so LinkedIn's compressor doesn't crush flat blacks."""
    tile = _noise_tile()
    noise = Image.new("RGB", img.size, (128, 128, 128))
    for y in range(0, img.size[1], 256):
        for x in range(0, img.size[0], 256):
            noise.paste(tile, (x, y))
    return Image.blend(img, noise, 0.05)


def new_canvas(flash: float = 0.0) -> Image.Image:
    base = tuple(int(lerp(c, 255, flash * 0.35)) for c in BG) if flash else BG
    if flash > 0.55:
        base = tuple(int(lerp(TEAL[i], 255, (flash - 0.55) * 0.5)) for i in range(3))
    img = Image.new("RGB", (W, H), base)
    draw = ImageDraw.Draw(img)
    # asymmetric signal grid
    for y in range(0, H, 90):
        draw.line([(0, y), (W, y)], fill=(14, 18, 26), width=1)
    for x in range(0, W, 140):
        draw.line([(x, 0), (x, H)], fill=(12, 16, 24), width=1)
    # hot left rail + corner ticks
    draw.rectangle([0, 0, 22, H], fill=TEAL)
    draw.rectangle([W - 22, 0, W, H], fill=(20, 28, 34))
    draw.rectangle([0, 0, W, 10], fill=TEAL if flash > 0.2 else (18, 28, 32))
    draw.rectangle([0, H - 10, W, H], fill=(18, 28, 32))
    return img


def draw_rec(draw: ImageDraw.ImageDraw, t: float, x: int = 120, y: int = 80) -> None:
    on = int(t * 2) % 2 == 0
    if on:
        draw.ellipse([x, y, x + 44, y + 44], fill=REC_RED)
    else:
        draw.ellipse([x, y, x + 44, y + 44], outline=REC_RED, width=4)
    draw.text((x + 64, y - 4), "REC  LIVE", font=font(FONT_NARROW, 54), fill=REC_RED)


def draw_waveform(
    draw: ImageDraw.ImageDraw,
    cx: int,
    cy: int,
    t: float,
    amp: float = 1.0,
    bars: int = 48,
    gap: int = 22,
    bar_w: int = 12,
) -> None:
    total_w = bars * gap
    x0 = cx - total_w // 2
    for i in range(bars):
        phase = t * 9.0 + i * 0.42
        envelope = 0.35 + 0.65 * abs(math.sin(i * 0.31 + t * 1.7))
        h = int((30 + 160 * abs(math.sin(phase)) * amp) * envelope)
        x = x0 + i * gap
        color = TEAL_HOT if i % 4 == 0 else TEAL
        draw.rounded_rectangle([x, cy - h, x + bar_w, cy + h], radius=5, fill=color)


def centered_text(
    draw: ImageDraw.ImageDraw,
    text: str,
    y: int,
    fnt: ImageFont.FreeTypeFont,
    fill: tuple[int, int, int],
    x_offset: int = 0,
) -> None:
    bbox = draw.textbbox((0, 0), text, font=fnt)
    tw = bbox[2] - bbox[0]
    draw.text(((W - tw) // 2 + x_offset, y), text, font=fnt, fill=fill)


def slam_text(
    img: Image.Image,
    text: str,
    y: int,
    size: int,
    t_local: float,
    fill: tuple[int, int, int] = INK,
    font_path: str = FONT_DISPLAY,
) -> None:
    """Scale-in slam for scroll-stop hooks."""
    if t_local <= 0:
        return
    progress = ease_out_back(min(1.0, t_local / 0.35))
    scale = lerp(1.28, 1.0, progress)
    fade = min(1.0, t_local / 0.10)
    fnt = font(font_path, max(24, int(size * scale)))
    overlay = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    od = ImageDraw.Draw(overlay)
    bbox = od.textbbox((0, 0), text, font=fnt)
    tw = bbox[2] - bbox[0]
    x = (W - tw) // 2
    for dx, dy, col in (
        (10, 12, (0, 0, 0, int(180 * fade))),
        (-5, 0, (*TEAL, int(90 * fade))),
        (5, 0, (*TEAL, int(50 * fade))),
    ):
        od.text((x + dx, y + dy), text, font=fnt, fill=col)
    od.text((x, y), text, font=fnt, fill=(*fill, int(255 * fade)))
    base = img.convert("RGBA")
    composed = Image.alpha_composite(base, overlay).convert("RGB")
    img.paste(composed)


def save_frame(img: Image.Image, path: Path) -> None:
    add_grain(img).save(path, "PNG", optimize=False)


def render_intro(frames_dir: Path, logo: Path | None, seconds: float = 11.0) -> int:
    """Punchy cold-open: hook → claim → brand → terminal type."""
    frames_dir.mkdir(parents=True, exist_ok=True)
    n = int(seconds * FPS)
    logo_img = None
    if logo and logo.exists():
        logo_img = Image.open(logo).convert("RGBA").resize((260, 260), Image.Resampling.LANCZOS)

    for i in range(n):
        t = i / FPS
        flash = 0.0
        if 2.15 < t < 2.28 or 4.35 < t < 4.48 or 7.05 < t < 7.18:
            flash = 1.0
        img = new_canvas(flash=flash)
        draw = ImageDraw.Draw(img)

        # --- ACT 1: cold open ---
        if t < 2.2:
            draw_rec(draw, t)
            local = t
            # giant single word
            slam_text(img, "HABLA.", 720, 320, local, fill=INK, font_path=FONT_DISPLAY)
            if t > 0.7:
                fade = ease_out_cubic((t - 0.7) / 0.4)
                fnt = font(FONT_NARROW, 72)
                centered_text(
                    draw,
                    "al agente. en la terminal. sin salir del TUI.",
                    1120,
                    fnt,
                    tuple(int(MUTED[j] * fade + BG[j] * (1 - fade)) for j in range(3)),
                )
            draw_waveform(draw, W // 2, 1550, t, amp=0.4 + 0.9 * min(1.0, t / 0.8), bars=56)

        # --- ACT 2: claim ---
        elif t < 4.4:
            local = t - 2.2
            draw_rec(draw, t)
            slam_text(img, "WHISPER LOCAL", 560, 200, local, fill=TEAL_HOT, font_path=FONT_DISPLAY)
            if local > 0.35:
                slam_text(
                    img,
                    "→ CURSOR / OPENCODE",
                    860,
                    150,
                    local - 0.35,
                    fill=INK,
                    font_path=FONT_NARROW,
                )
            if local > 0.7:
                centered_text(
                    draw,
                    "sin API key de speech  ·  confirmas antes de enviar",
                    1180,
                    font(FONT_MONO, 52),
                    MUTED,
                )
            draw_waveform(draw, W // 2, 1600, t, amp=1.1, bars=52)

        # --- ACT 3: brand slam ---
        elif t < 7.1:
            local = t - 4.4
            if logo_img:
                # logo flies in from left
                lx = int(lerp(-200, 220, ease_out_cubic(min(1.0, local / 0.35))))
                img.paste(logo_img, (lx, 280), logo_img)
            slam_text(img, "TERMVOX", 300, 260, local, fill=INK, font_path=FONT_DISPLAY)
            # alpha chip
            chip = font(FONT_MONO, 48)
            draw.rounded_rectangle([240, 620, 700, 710], radius=16, fill=(18, 40, 38), outline=TEAL, width=4)
            draw.text((280, 638), "alpha preview", font=chip, fill=TEAL_HOT)
            if local > 0.4:
                centered_text(
                    draw,
                    "la capa de voz que faltaba en tu agent CLI",
                    820,
                    font(FONT_NARROW, 78),
                    INK,
                )
            # three punch bullets
            if local > 0.7:
                bullets = [
                    ("01", "F8 / Ctrl+Space"),
                    ("02", "Whisper on-device"),
                    ("03", "mismo TUI del agente"),
                ]
                for bi, (num, label) in enumerate(bullets):
                    bx = 420 + bi * 1050
                    by = 1100
                    draw.rounded_rectangle([bx, by, bx + 920, by + 220], radius=24, fill=PANEL, outline=TEAL, width=3)
                    draw.text((bx + 48, by + 40), num, font=font(FONT_MONO_BLACK, 56), fill=TEAL)
                    draw.text((bx + 48, by + 120), label, font=font(FONT_NARROW, 58), fill=INK)

        # --- ACT 4: terminal tease ---
        else:
            local = t - 7.1
            draw_rec(draw, t)
            draw.text((120, 160), "TERMVOX", font=font(FONT_DISPLAY, 72), fill=TEAL)
            draw.text((520, 185), "shell session", font=font(FONT_MONO, 44), fill=MUTED)

            px, py, pw, ph = 220, 320, W - 440, 1480
            draw.rounded_rectangle([px, py, px + pw, py + ph], radius=32, fill=PANEL, outline=TEAL, width=4)
            # traffic lights
            for cx, col in ((px + 70, DANGER), (px + 130, (255, 189, 46)), (px + 190, (39, 201, 63))):
                draw.ellipse([cx, py + 48, cx + 36, py + 84], fill=col)
            draw.text((px + 260, py + 44), "~/proyecto", font=font(FONT_MONO_REG, 48), fill=DIM)

            line1 = "npm install -g termvox"
            line2 = "termvox shell --agent opencode"
            typed1 = min(len(line1), max(0, int(local * 22)))
            typed2 = min(len(line2), max(0, int((local - 1.4) * 20)))
            cmd_f = font(FONT_MONO, 72)
            y1 = py + 280
            draw.text((px + 100, y1), "$ ", font=cmd_f, fill=TEAL)
            chunk1 = line1[:typed1]
            draw.text((px + 180, y1), chunk1, font=cmd_f, fill=INK)
            if typed1 < len(line1) and int(t * 5) % 2 == 0:
                cx1 = px + 180 + int(draw.textlength(chunk1, font=cmd_f))
                draw.rectangle([cx1, y1 + 10, cx1 + 36, y1 + 70], fill=TEAL)

            if local > 1.2:
                draw.text((px + 100, y1 + 140), "✓ ready in seconds", font=font(FONT_MONO, 48), fill=TEAL_HOT)

            y2 = y1 + 320
            draw.text((px + 100, y2), "$ ", font=cmd_f, fill=TEAL)
            chunk2 = line2[: max(0, typed2)]
            draw.text((px + 180, y2), chunk2, font=cmd_f, fill=INK)
            if local > 1.4 and typed2 < len(line2) and int(t * 5) % 2 == 0:
                cx2 = px + 180 + int(draw.textlength(chunk2, font=cmd_f))
                draw.rectangle([cx2, y2 + 10, cx2 + 36, y2 + 70], fill=TEAL)

            if local > 2.6:
                draw_waveform(draw, W // 2, py + ph - 220, t, amp=1.25, bars=44)
                centered_text(
                    draw,
                    "AHORA MIRA LO QUE PASA EN EL TUI →",
                    py + ph - 80,
                    font(FONT_NARROW, 56),
                    TEAL_HOT,
                )

        save_frame(img, frames_dir / f"frame_{i:05d}.png")
    return n


def render_outro(frames_dir: Path, logo: Path | None, seconds: float = 6.0) -> int:
    frames_dir.mkdir(parents=True, exist_ok=True)
    n = int(seconds * FPS)
    logo_img = None
    if logo and logo.exists():
        logo_img = Image.open(logo).convert("RGBA").resize((200, 200), Image.Resampling.LANCZOS)

    for i in range(n):
        t = i / FPS
        flash = 1.0 if t < 0.08 else 0.0
        img = new_canvas(flash=flash)
        draw = ImageDraw.Draw(img)
        if logo_img:
            img.paste(logo_img, (200, 220), logo_img)

        slam_text(img, "PRUÉBALO HOY", 200, 180, t, fill=INK, font_path=FONT_DISPLAY)
        if t > 0.35:
            centered_text(draw, "alpha abierta  ·  feedback welcome", 470, font(FONT_NARROW, 64), MUTED)

        # giant command block
        draw.rounded_rectangle([260, 620, W - 260, 1180], radius=36, fill=PANEL, outline=TEAL, width=5)
        draw.text((380, 720), "$ npm install -g termvox", font=font(FONT_MONO_BLACK, 84), fill=INK)
        draw.text((380, 900), "$ termvox shell --agent opencode", font=font(FONT_MONO, 72), fill=TEAL_HOT)

        if t > 0.8:
            centered_text(draw, "github.com/Jeronimo0228/termvox", 1320, font(FONT_NARROW, 72), INK)
            centered_text(draw, "npmjs.com/package/termvox", 1440, font(FONT_MONO, 48), MUTED)

        draw_waveform(draw, W // 2, 1780, t, amp=0.9 + 0.3 * math.sin(t * 4), bars=50)
        centered_text(
            draw,
            "VOZ LOCAL  ·  CURSOR  ·  OPENCODE  ·  CLAUDE  ·  CODEX",
            1980,
            font(FONT_NARROW, 48),
            DIM,
        )
        save_frame(img, frames_dir / f"frame_{i:05d}.png")
    return n


def x264_args(kind: str) -> list[str]:
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
            "32M",
            "-maxrate",
            "40M",
            "-bufsize",
            "64M",
            "-x264-params",
            "aq-mode=3:aq-strength=1.2",
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
        "14M",
        "-maxrate",
        "18M",
        "-bufsize",
        "28M",
        "-x264-params",
        "aq-mode=3:aq-strength=1.2",
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
    end = ass_time(max(1.0, duration - 0.05))
    # Billboard captions — huge, short, high contrast
    content = f"""[Script Info]
Title: TermVox LinkedIn Punch
ScriptType: v4.00+
PlayResX: {W}
PlayResY: {H}
WrapStyle: 0

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Billboard,Liberation Sans Narrow,{110},&H00F0F0E8,&H000000FF,&H00100000,&HCC000000,-1,0,0,0,100,100,0,0,3,0,0,2,60,60,140,1
Style: Tag,Source Code Pro,{58},&H00C6E800,&H000000FF,&H00100000,&HDD000000,-1,0,0,0,100,100,0,0,3,0,0,7,70,70,70,1
Style: Hot,Libre Franklin,{96},&H00C6E800,&H000000FF,&H00100000,&HDD000000,-1,0,0,0,100,100,0,0,3,0,0,8,70,70,180,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:03.20,Tag,,0,0,0,,● TERMVOX LIVE
Dialogue: 0,0:00:00.15,0:00:03.40,Hot,,0,0,0,,MIC BAR INTEGRADA EN EL TUI
Dialogue: 0,0:00:03.40,0:00:08.50,Billboard,,0,0,0,,F8 = HABLAR  ·  Whisper on-device
Dialogue: 0,0:00:08.50,0:00:16.00,Billboard,,0,0,0,,PROMPT POR VOZ → EL AGENTE LO RECIBE
Dialogue: 0,0:00:16.00,0:00:28.00,Billboard,,0,0,0,,OPENCODE TRABAJANDO EN EL MISMO TERMINAL
Dialogue: 0,0:00:28.00,0:00:42.00,Billboard,,0,0,0,,SIN SALIR  ·  SIN PEGAR  ·  SIN API DE SPEECH
Dialogue: 0,0:00:42.00,{end},Billboard,,0,0,0,,ALPHA  ·  github.com/Jeronimo0228/termvox
"""
    path.write_text(content, encoding="utf-8")


def process_source(source: Path, ass: Path, out_mp4: Path) -> None:
    pad_color = f"0x{BG[0]:02x}{BG[1]:02x}{BG[2]:02x}"
    # Punch: slight continuous zoom + vignette + teal frame + captions
    vf = (
        f"fps={FPS},"
        f"scale={W}:{H}:force_original_aspect_ratio=decrease:flags=lanczos,"
        f"pad={W}:{H}:(ow-iw)/2:(oh-ih)/2:{pad_color},"
        "zoompan=z='min(1.08,1.0+0.00035*on)':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d=1:s=3840x2160:fps=30,"
        "unsharp=5:5:0.8:5:5:0.0,"
        "drawbox=x=0:y=0:w=28:h=ih:color=0x00E8C6@1:t=fill,"
        "drawbox=x=0:y=0:w=iw:h=14:color=0x00E8C6@0.85:t=fill,"
        "drawbox=x=0:y=ih-14:w=iw:h=14:color=0x00E8C6@0.55:t=fill,"
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

    print()
    print("Wrote:")
    print(f"  {master}")
    print(f"  {linkedin_1080}")
    print("Upload the 1080p file to LinkedIn for best in-feed results.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
