"""Exercise a real sprite editor with a technical fixture, not character artwork."""

import argparse
import json
import os
from pathlib import Path
import struct
import subprocess
import zlib


def require(condition, message):
    if not condition:
        raise ValueError(message)


def string(value):
    data = value.encode("utf-8")
    return struct.pack("<H", len(data)) + data


def chunk(kind, data):
    return struct.pack("<IH", len(data) + 6, kind) + data


def fixture(path, colours):
    layers = []
    for name in ("body", "main_hand", "off_hand"):
        data = struct.pack("<HHHHHHB", 1, 0, 0, 0, 0, 0, 255) + bytes(3) + string(name)
        layers.append(chunk(0x2004, data))
    tags = struct.pack("<H", 2) + bytes(8)
    for start, end, name in ((0, 1, "idle_left"), (2, 3, "idle_right")):
        tags += struct.pack("<HHB", start, end, 0) + bytes(8) + bytes([128, 128, 128, 0]) + string(name)
    frames = []
    for frame, duration in enumerate((100, 180, 120, 220)):
        chunks = layers + [chunk(0x2018, tags)] if frame == 0 else []
        for layer in range(3):
            pixels = bytearray(64 * 64 * 4)
            x = (24 if frame < 2 else 35) + layer * 3
            y = 26 + layer * 7 + frame % 2
            for yy in range(y, y + 5):
                for xx in range(x, x + 3):
                    offset = (yy * 64 + xx) * 4
                    pixels[offset:offset + 4] = bytes(colours[layer] + [255])
            cel = struct.pack("<HhhBH", layer, 0, 0, 255, 2) + bytes(7)
            cel += struct.pack("<HH", 64, 64) + zlib.compress(pixels)
            chunks.append(chunk(0x2005, cel))
        content = b"".join(chunks)
        frames.append(struct.pack("<IHHHHI", len(content) + 16, 0xF1FA, len(chunks), duration, 0, 0) + content)
    header = bytearray(128)
    struct.pack_into("<IHHHHHIH", header, 0, 128 + sum(map(len, frames)), 0xA5E0, 4, 64, 64, 32, 1, 100)
    header[34:36] = bytes([1, 1])
    path.write_bytes(header + b"".join(frames))


def inspect_source(path):
    data = path.read_bytes()
    require(struct.unpack_from("<H", data, 4)[0] == 0xA5E0, "Invalid sprite source")
    count, width, height, depth = struct.unpack_from("<HHHH", data, 6)
    durations, names = [], []
    offset = 128
    for _ in range(count):
        size, magic, chunks, duration, _, new_count = struct.unpack_from("<IHHHHI", data, offset)
        require(magic == 0xF1FA, "Invalid frame header")
        durations.append(duration)
        cursor = offset + 16
        for _ in range(new_count or chunks):
            length, kind = struct.unpack_from("<IH", data, cursor)
            if kind == 0x2004:
                name_length = struct.unpack_from("<H", data, cursor + 22)[0]
                names.append(data[cursor + 24:cursor + 24 + name_length].decode())
            cursor += length
        offset += size
    return {"size": [width, height], "depth": depth, "durations": durations, "layers": names}


def png_pixels(path):
    data = path.read_bytes()
    require(data[:8] == b"\x89PNG\r\n\x1a\n", "Invalid PNG")
    offset, compressed, palette, alpha = 8, b"", b"", b""
    while offset < len(data):
        length = struct.unpack_from(">I", data, offset)[0]
        kind, payload = data[offset + 4:offset + 8], data[offset + 8:offset + 8 + length]
        if kind == b"IHDR":
            width, height, bits, colour, compression, filtering, interlace = struct.unpack(">IIBBBBB", payload)
            require(bits == 8 and colour in (2, 3, 6) and not interlace, "Unsupported PNG encoding")
        elif kind == b"IDAT":
            compressed += payload
        elif kind == b"PLTE":
            palette = payload
        elif kind == b"tRNS":
            alpha = payload
        offset += length + 12
    channels = {2: 3, 3: 1, 6: 4}[colour]
    stride, raw, previous, result = width * channels, zlib.decompress(compressed), bytearray(width * channels), []
    for y in range(height):
        start = y * (stride + 1)
        method, row = raw[start], bytearray(raw[start + 1:start + 1 + stride])
        require(method <= 4, "Invalid PNG filter")
        for x in range(stride):
            left = row[x - channels] if x >= channels else 0
            up = previous[x]
            corner = previous[x - channels] if x >= channels else 0
            if method == 1:
                predictor = left
            elif method == 2:
                predictor = up
            elif method == 3:
                predictor = (left + up) // 2
            elif method == 4:
                p = left + up - corner
                distances = [abs(p - left), abs(p - up), abs(p - corner)]
                predictor = [left, up, corner][distances.index(min(distances))]
            else:
                predictor = 0
            row[x] = (row[x] + predictor) % 256
        for x in range(0, stride, channels):
            if colour == 3:
                index = row[x]
                result.append(tuple(palette[index * 3:index * 3 + 3]) + (alpha[index] if index < len(alpha) else 255,))
            else:
                result.append(tuple(row[x:x + channels]) + (() if channels == 4 else (255,)))
        previous = row
    return width, height, result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--editor", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path, help="New directory for technical test artifacts")
    args = parser.parse_args()
    require(args.editor.is_file(), "Editor executable is missing")
    args.output.mkdir(parents=True, exist_ok=False)
    root = Path(__file__).resolve().parents[2]
    ramps = json.loads((root / "docs/style/gravebound-palette-candidate.json").read_text())["ramps"]
    colours = [list(bytes.fromhex(ramps[name][3][1:])) for name in ("charcoal", "steel", "leather")]
    source = args.output / "technical_fixture.aseprite"
    fixture(source, colours)
    editor = str(args.editor.resolve())
    config = args.output / "editor_config"
    config.mkdir()
    env = {**os.environ, "SDL_VIDEODRIVER": "dummy", "SDL_AUDIODRIVER": "dummy",
           "XDG_CONFIG_HOME": str(config.resolve())}

    def run(*options):
        try:
            subprocess.run([editor, "--batch", *map(str, options)], check=True, env=env, capture_output=True, text=True)
        except subprocess.CalledProcessError as error:
            raise RuntimeError(f"Editor failed:\n{error.stdout}\n{error.stderr}") from error

    saved = args.output / "reopened.aseprite"
    run(source, "--save-as", saved)
    require(inspect_source(source) == inspect_source(saved), "Source round-trip changed layers, dimensions or timing")
    atlases = []
    for name, file in (("original", source), ("reopened", saved)):
        atlas, metadata = args.output / f"{name}.png", args.output / f"{name}.json"
        run("--list-layers", "--list-tags", file, "--sheet-type", "horizontal",
            "--format", "json-array", "--sheet", atlas, "--data", metadata)
        meta = json.loads(metadata.read_text())
        frames = meta["frames"]
        require([f["duration"] for f in frames] == [100, 180, 120, 220], "Export lost frame durations")
        require(all(f["sourceSize"] == {"w": 64, "h": 64} and not f["trimmed"] for f in frames), "Export trimmed source cells")
        require([(t["name"], t["from"], t["to"]) for t in meta["meta"]["frameTags"]] ==
                [("idle_left", 0, 1), ("idle_right", 2, 3)], "Export lost facing tags")
        width, height, pixels = png_pixels(atlas)
        require((width, height) == (256, 64), "Unexpected atlas dimensions")
        visible = [p for p in pixels if p[3]]
        require(len(visible) == 4 * 3 * 15, "Missing or unexpected layer pixels")
        require(set(visible) == {tuple(c + [255]) for c in colours}, "Export changed colours or alpha")
        require(pixels[0][3] == 0, "Transparent border lost")
        atlases.append(pixels)
    require(atlases[0] == atlases[1], "Round-trip changed composited pixels")
    for layer, colour in zip(("body", "main_hand", "off_hand"), colours):
        atlas = args.output / f"{layer}.png"
        run("--layer", layer, saved, "--sheet-type", "horizontal", "--sheet", atlas)
        _, _, pixels = png_pixels(atlas)
        require({p for p in pixels if p[3]} == {tuple(colour + [255])}, f"Layer isolation failed: {layer}")
    report = {"status": "passed", "editor": subprocess.run([editor, "--version"], env=env, check=True,
              capture_output=True, text=True).stdout.strip().splitlines()[0], "frames": 4, "layers": 3,
              "atlas": [256, 64], "palette": "candidate subset only",
              "art_quality_approved": False, "bevy_integration_tested": False}
    (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
