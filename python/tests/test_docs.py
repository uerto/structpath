import re
import sys
import traceback
from collections.abc import Iterator
from pathlib import Path

import pytest


def code_blocks(path: Path) -> Iterator[tuple[int, int, str]]:
    pattern = re.compile(
        r"""
        (?:\n+|\A\n?|(?<=\n))
        (^[ \t]*`{3,})\s{0,99}?([\w+-]+)?\s{0,99}?\n
        (.*?)                             # $3 = code block content
        \1[ \t]*\n                      # closing fence
        """,
        re.M | re.X | re.S,
    )
    with open(path, "r") as f:
        file = f.read()

    for match in pattern.finditer(file):
        if not match[2]:
            continue
        if match[2].lower() == "python":
            block = match[3]
            block_index = file.index(block)
            start = file[:block_index].count("\n")
            end = start + block.count("\n")
            yield start, end, block


blocks = [
    (path, block, start, end)
    for path in Path("docs").glob("**/*.md")
    for start, end, block in code_blocks(path)
]


@pytest.mark.parametrize("path, block,start, end", blocks)
def test_block(path, block, start, end):
    # remove blockwise leading spaces
    lines = block.split("\n")
    while all(line.startswith(" ") or not line for line in lines):
        lines = [line[1:] for line in lines]
    block = "\n".join(lines)

    try:
        exec(block, globals())
    except Exception as error:
        _, _, tb = sys.exc_info()
        line_in_block = traceback.extract_tb(tb)[-1][1]
        raise Exception(
            f"{path}, line: {line_in_block + start}: {error.__class__.__name__} {error}"
        ) from error
