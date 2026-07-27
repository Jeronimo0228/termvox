#!/usr/bin/env python3
"""Create a deterministic TermVox release archive."""

from __future__ import annotations

import gzip
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import zipfile


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(2)


if len(sys.argv) != 3:
    fail("usage: reproducible-archive.py BINARY OUTPUT.{tar.gz|zip}")

source = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
repo = Path(__file__).resolve().parent.parent
if not source.is_file():
    fail(f"binary does not exist: {source}")

try:
    epoch = int(os.environ.get("SOURCE_DATE_EPOCH", "0"))
except ValueError:
    fail("SOURCE_DATE_EPOCH must be an integer")

output.parent.mkdir(parents=True, exist_ok=True)
binary_name = "termvox.exe" if output.suffix == ".zip" else "termvox"
shell_extensions = {
    "bash": "bash",
    "zsh": "zsh",
    "fish": "fish",
    "powershell": "ps1",
    "elvish": "elv",
}


def stage_release(directory: Path) -> None:
    binary = directory / binary_name
    shutil.copyfile(source, binary)
    binary.chmod(0o755)
    for name in ("README.md", "LICENSE-MIT", "LICENSE-APACHE"):
        shutil.copyfile(repo / name, directory / name)
    subprocess.run(
        [str(source), "manpage", "--output", str(directory / "termvox.1")],
        check=True,
    )
    completions = directory / "completions"
    completions.mkdir()
    for shell, extension in shell_extensions.items():
        with (completions / f"termvox.{extension}").open("wb") as destination:
            subprocess.run(
                [str(source), "completions", shell],
                stdout=destination,
                check=True,
            )


def archive_paths(directory: Path) -> list[Path]:
    return sorted(
        (path for path in directory.rglob("*") if path.is_file()),
        key=lambda path: path.relative_to(directory).as_posix(),
    )


with tempfile.TemporaryDirectory() as temp_dir:
    staged = Path(temp_dir)
    stage_release(staged)
    paths = archive_paths(staged)

    if output.name.endswith(".tar.gz"):
        with output.open("wb") as raw:
            with gzip.GzipFile(
                filename="", mode="wb", fileobj=raw, mtime=epoch
            ) as compressed:
                with tarfile.open(fileobj=compressed, mode="w") as archive:
                    for path in paths:
                        relative = path.relative_to(staged).as_posix()
                        info = archive.gettarinfo(str(path), arcname=relative)
                        info.uid = 0
                        info.gid = 0
                        info.uname = ""
                        info.gname = ""
                        info.mtime = epoch
                        info.mode = 0o755 if path.name == binary_name else 0o644
                        with path.open("rb") as contents:
                            archive.addfile(info, contents)
    elif output.suffix == ".zip":
        zip_epoch = max(epoch, 315532800)
        date_time = time.gmtime(zip_epoch)[:6]
        with zipfile.ZipFile(output, "w") as archive:
            for path in paths:
                relative = path.relative_to(staged).as_posix()
                info = zipfile.ZipInfo(relative, date_time=date_time)
                info.compress_type = zipfile.ZIP_DEFLATED
                info.create_system = 3
                mode = 0o755 if path.name == binary_name else 0o644
                info.external_attr = mode << 16
                archive.writestr(info, path.read_bytes())
    else:
        fail("output must end in .tar.gz or .zip")
