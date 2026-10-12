#!/usr/bin/env python3
"""Turn a failed Android build log into GitHub Actions error annotations.

Usage: report_errors.py <build.log>

Each compiler or tool error block (from an ``error`` / ``Error:`` line to the next blank line,
at most 60 lines) becomes one ``::error::`` annotation, followed by one with the end of the log,
so the cause of a failure can be read in the job summary or through the checks API without
downloading the whole log. Always exits 0; the build step already failed the job.
"""

import re
import sys

MAX_BLOCKS = 8
BLOCK_LINES = 60
TAIL_LINES = 80

START = re.compile(r"^(error(\[E\d+\])?:|error: |Error:|thread '.*' panicked|fatal:|.*: error: )")
ANSI = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")


def escape(text: str) -> str:
    return text.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")


def annotate(title: str, lines: list[str]) -> None:
    print(f"::error title={escape(title)}::{escape(chr(10).join(lines))}")


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: report_errors.py <build.log>", file=sys.stderr)
        return 0
    try:
        with open(sys.argv[1], encoding="utf-8", errors="replace") as f:
            lines = [ANSI.sub("", line.rstrip("\n")) for line in f]
    except OSError as e:
        print(f"::error title=Android build::no build log: {e}")
        return 0

    blocks: list[list[str]] = []
    i = 0
    while i < len(lines) and len(blocks) < MAX_BLOCKS:
        if START.match(lines[i]) and "aborting due to" not in lines[i]:
            block = []
            while i < len(lines) and lines[i].strip() and len(block) < BLOCK_LINES:
                block.append(lines[i])
                i += 1
            blocks.append(block)
        i += 1

    for n, block in enumerate(blocks, 1):
        annotate(f"Android build error {n}", block)
    annotate("Android build log (end)", lines[-TAIL_LINES:])
    return 0


if __name__ == "__main__":
    sys.exit(main())
