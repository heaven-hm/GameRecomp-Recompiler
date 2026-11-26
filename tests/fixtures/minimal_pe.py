#!/usr/bin/env python3
"""Build a deterministic, minimal PE32 x86 executable for CLI inspect tests.

This is NOT a recompiler, loader or lifter. It only emits the few header bytes
that ``game-recomp-cli inspect`` reads so that the real CLI can be exercised
without shipping a copyrighted game binary. No code or sections are executed;
the resulting file is only valid enough for header inspection.

Usage:
    python3 tests/fixtures/minimal_pe.py OUTPUT.exe

With no OUTPUT argument the bytes are written to stdout (binary).
"""

from __future__ import annotations

import struct
import sys

# The inspect command in GameReComp (src/pe.rs) only reads the DOS header, the
# PE signature, the COFF machine field and the PE32 optional-header magic and
# ImageBase. These values make ``ok=true`` and ``pe32_x86=true``.
PE_OFFSET = 0x80
FILE_SIZE = 0x100  # PE_OFFSET + 0x40 header block + slack
IMAGE_BASE = 0x00400000
MACHINE_I386 = 0x014C
MAGIC_PE32 = 0x010B


def build_minimal_pe32() -> bytes:
    """Return a fixed, reproducible PE32 x86 byte image (zero TimeDateStamp)."""
    data = bytearray(FILE_SIZE)

    # --- MS-DOS header / stub ---------------------------------------------
    data[0:2] = b"MZ"  # e_magic
    struct.pack_into("<H", data, 0x02, 0x0090)  # e_cblp
    struct.pack_into("<H", data, 0x04, 0x0003)  # e_cp
    struct.pack_into("<H", data, 0x08, 0x0004)  # e_cparhdr
    struct.pack_into("<H", data, 0x0A, 0x0000)  # e_minalloc
    struct.pack_into("<H", data, 0x0C, 0xFFFF)  # e_maxalloc
    struct.pack_into("<H", data, 0x10, 0x00B8)  # e_ss
    struct.pack_into("<H", data, 0x14, 0x0000)  # e_sp
    struct.pack_into("<I", data, 0x3C, PE_OFFSET)  # e_lfanew

    # --- PE signature + COFF file header (pe_off .. pe_off+0x18) ----------
    pe = PE_OFFSET
    data[pe : pe + 4] = b"PE\0\0"
    struct.pack_into("<H", data, pe + 4, MACHINE_I386)  # Machine
    struct.pack_into("<H", data, pe + 6, 1)  # NumberOfSections
    struct.pack_into("<I", data, pe + 8, 0)  # TimeDateStamp (deterministic)
    struct.pack_into("<H", data, pe + 20, 0x0102)  # executable + 32-bit

    # --- PE32 optional header (starts at pe_off + 24) ---------------------
    opt = pe + 24
    struct.pack_into("<H", data, opt + 0, MAGIC_PE32)  # Magic
    struct.pack_into("<I", data, opt + 28, IMAGE_BASE)  # ImageBase (PE32)
    struct.pack_into("<I", data, opt + 32, 0x1000)  # SectionAlignment
    struct.pack_into("<I", data, opt + 36, 0x200)  # FileAlignment
    struct.pack_into("<H", data, opt + 68, 3)  # Subsystem = console

    return bytes(data)


def main(argv: list[str]) -> int:
    if len(argv) > 2:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    image = build_minimal_pe32()
    if len(argv) == 2:
        with open(argv[1], "wb") as handle:
            handle.write(image)
    else:
        sys.stdout.buffer.write(image)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
