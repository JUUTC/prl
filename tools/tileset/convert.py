#!/usr/bin/env python3
"""Convert the pinned, audited UltiCa crypt subset; normal builds need no network."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import stat
import zipfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUTPUT = ROOT / "assets/packs/prl/sprites/ultica"
SELECTION = HERE / "ultica.json"
PREFIX = "UltimateCataclysm/"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def safe_path(name: str) -> str:
    path = PurePosixPath(name)
    if not name or "\\" in name or ":" in name or path.is_absolute() or ".." in path.parts:
        raise ValueError(f"unsafe asset path: {name!r}")
    return name


class Tileset:
    """Resolve global indices with sheet-specific bounds, dimensions and offsets."""

    def __init__(self, data: bytes, expected_hash: str):
        from PIL import Image

        if digest(data) != expected_hash:
            raise ValueError("archive SHA-256 does not match the pinned release")
        self.archive = zipfile.ZipFile(io.BytesIO(data))
        total = 0
        names = set()
        for entry in self.archive.infolist():
            safe_path(entry.filename)
            if entry.filename in names or stat.S_ISLNK(entry.external_attr >> 16):
                raise ValueError(f"duplicate or symlink archive entry: {entry.filename}")
            names.add(entry.filename)
            total += entry.file_size
        if total > 128 * 1024 * 1024:
            raise ValueError("tileset exceeds the 128 MiB input budget")
        config = json.loads(self.archive.read(PREFIX + "tile_config.json"))
        base = config["tile_info"][0]
        if (base["width"], base["height"], base.get("iso", False)) != (32, 32, False):
            raise ValueError("expected non-isometric 32x32 source cells")
        self.sheets = []
        self.entries = {}
        start = 0
        for sheet in config["tiles-new"]:
            name = safe_path(sheet["file"])
            image = Image.open(io.BytesIO(self.archive.read(PREFIX + name))).convert("RGBA")
            width = sheet.get("sprite_width", 32)
            height = sheet.get("sprite_height", 32)
            if width <= 0 or height <= 0 or image.width % width or image.height % height:
                raise ValueError(f"invalid sheet dimensions: {name}")
            count = (image.width // width) * (image.height // height)
            self.sheets.append((start, start + count, sheet, image, width, height))
            start += count
            for entry in sheet.get("tiles", []):
                ids = entry["id"] if isinstance(entry["id"], list) else [entry["id"]]
                for tile_id in ids:
                    # The upstream release contains overrides; reject ambiguity only
                    # for selected IDs rather than silently choosing a definition.
                    self.entries.setdefault(tile_id, []).append(entry)

    def layer(self, tile_id: str, plane: str, subtile: str | None, variant: int):
        entries = self.entries.get(tile_id, [])
        if len(entries) != 1:
            raise ValueError(f"missing or ambiguous selected tile: {tile_id}")
        entry = entries[0]
        if entry.get("animated", False):
            raise ValueError(f"animated selected tile is unsupported: {tile_id}")
        if subtile is not None:
            matches = [t for t in entry.get("additional_tiles", []) if t["id"] == subtile]
            if len(matches) != 1:
                raise ValueError(f"missing or ambiguous subtile: {tile_id}/{subtile}")
            entry = matches[0]
        value = entry[plane]
        if isinstance(value, list):
            if not all(isinstance(v, dict) and v.get("weight", 0) > 0 for v in value):
                raise ValueError(f"directional/unknown array is unsupported: {tile_id}")
            if not 0 <= variant < len(value):
                raise ValueError(f"variant out of range: {tile_id}")
            index = value[variant]["sprite"]
        else:
            if variant != 0:
                raise ValueError(f"variant specified for scalar: {tile_id}")
            index = value
        if type(index) is not int or index < 0:
            raise ValueError(f"invalid sprite index: {tile_id}: {index}")
        for start, end, sheet, image, width, height in self.sheets:
            if start <= index < end:
                column = (index - start) % (image.width // width)
                row = (index - start) // (image.width // width)
                rect = (column * width, row * height, (column + 1) * width, (row + 1) * height)
                offset = (sheet.get("sprite_offset_x", 0), sheet.get("sprite_offset_y", 0))
                return image.crop(rect), offset, {"sheet": sheet["file"], "index": index,
                                                "rect": list(rect), "offset": list(offset)}
        raise ValueError(f"sprite index outside all sheets: {tile_id}: {index}")


def composite(tileset: Tileset, layers: list, selection: dict, source_dir: Path):
    from PIL import Image

    canvas = Image.new("RGBA", tuple(selection["canvas"]))
    records = []
    for tile_id, plane, subtile, variant, source in layers:
        image, offset, record = tileset.layer(tile_id, plane, subtile, variant)
        path = source_dir / safe_path(source)
        raw = path.read_bytes()
        original = Image.open(io.BytesIO(raw)).convert("RGBA")
        # Compare visible pixels, disregarding RGB values behind fully clear alpha.
        def visible(picture):
            result = Image.new("RGBA", picture.size)
            result.alpha_composite(picture)
            return result.tobytes()
        if original.size != image.size or visible(original) != visible(image):
            raise ValueError(f"compiled sprite does not match audited source: {source}")
        x, y = (offset[i] - selection["origin"][i] for i in range(2))
        if x < 0 or y < 0 or x + image.width > canvas.width or y + image.height > canvas.height:
            raise ValueError(f"selected layer would be clipped: {source}")
        canvas.alpha_composite(image, (x, y))
        records.append(dict(record, tile_id=tile_id, plane=plane, subtile=subtile,
                            variant=variant, source=source, source_sha256=digest(raw)))
    return canvas, records


def generate(archive: bytes, selection: dict, source_dir: Path) -> dict[str, bytes]:
    from PIL import Image

    tileset = Tileset(archive, selection["archive_sha256"])
    frames = []
    entries = {}
    provenance = {}
    for visual_id, visual in sorted(selection["visuals"].items()):
        entry = {"kind": visual["kind"]}
        for key in ("layers", "open_layers"):
            if key not in visual:
                continue
            picture, records = composite(tileset, visual[key], selection, source_dir)
            entry["open" if key == "open_layers" else "rect"] = (
                len(frames) * picture.width, 0, picture.width, picture.height)
            frames.append(picture)
            provenance[f"{visual_id}/{key}"] = records
        entries[visual_id] = entry
    width, height = selection["canvas"]
    atlas = Image.new("RGBA", (width * len(frames), height))
    for index, frame in enumerate(frames):
        atlas.paste(frame, (index * width, 0))
    buffer = io.BytesIO()
    atlas.save(buffer, format="PNG", optimize=False, compress_level=9)
    origin_x, origin_y = selection["origin"]
    scale = selection["scale"]
    offset_x = (origin_x + width / 2 - 16) * scale
    offset_y = -(origin_y + height / 2 - 16) * scale
    lines = [
        "// Generated by tools/tileset/convert.py. CC BY-SA 3.0; see CREDITS.md.",
        "(",
        '    schema_version: 2, status: "candidate",',
        f'    source: "UltiCa {selection["release"]} / {selection["revision"]}",',
        f'    image: "ultica.png", size: ({atlas.width}, {atlas.height}), scale: {scale}.0,',
        "    actors: {",
    ]
    for visual_id, entry in entries.items():
        if entry["kind"] == "actor":
            lines.append(f'        "{visual_id}": (offset_x: {offset_x}, offset_y: {offset_y}, '
                         f'idle: (frame_ms: 1000, right: [{entry["rect"]}], left: None), '
                         'walk: None, attack: None),')
    lines += ["    },", "    tiles: {"]
    for visual_id, entry in entries.items():
        if entry["kind"] == "tile":
            opened = f'Some({entry["open"]})' if "open" in entry else "None"
            lines.append(f'        "{visual_id}": (rect: {entry["rect"]}, offset_x: {offset_x}, '
                         f'offset_y: {offset_y}, open: {opened}),')
    lines += ["    },", "    markers: {"]
    for visual_id, entry in entries.items():
        if entry["kind"] == "marker":
            lines.append(f'        "{visual_id.split(":")[-1]}": (rect: {entry["rect"]}, offset_x: {offset_x}, '
                         f'offset_y: {offset_y}, open: None),')
    lines += ["    },", ")"]
    outputs = {"ultica.png": buffer.getvalue(), "ultica.ron": ("\n".join(lines) + "\n").encode()}
    sources = sorted({r["source"] for rows in provenance.values() for r in rows})
    for source in sources:
        outputs["sources/" + source] = (source_dir / source).read_bytes()
    histories = json.loads((source_dir / "history.json").read_text())
    outputs["sources/history.json"] = (
        json.dumps({s: histories[s] for s in sources}, indent=2, sort_keys=True) + "\n").encode()
    outputs["selection.json"] = (json.dumps(selection, indent=2, sort_keys=True) + "\n").encode()
    report = {"revision": selection["revision"], "archive_sha256": digest(archive),
              "changes": "Selected fixed variants; composed layers; repacked native pixels. "
                         "Player kit is a fixed candidate study, not dynamic equipment.",
              "visuals": provenance, "source_history": {s: histories[s] for s in sources}}
    outputs["provenance.json"] = (json.dumps(report, indent=2, sort_keys=True) + "\n").encode()
    return outputs


def verify(output: Path) -> None:
    manifest = json.loads((output / "checksums.json").read_text())
    actual = {p.relative_to(output).as_posix() for p in output.rglob("*") if p.is_file()}
    if actual != set(manifest) | {"checksums.json"}:
        raise ValueError("candidate directory has missing or untracked files")
    for name, expected in manifest.items():
        if digest((output / safe_path(name)).read_bytes()) != expected:
            raise ValueError(f"packaged candidate file differs: {name}")
    required = {"ultica.png", "ultica.ron", "provenance.json", "selection.json",
                "sources/history.json", "CREDITS.md",
                "CC-BY-SA-3.0.txt", "upstream-readme.md", "upstream-LICENSE.txt"}
    if not required.issubset(manifest):
        raise ValueError("candidate checksum inventory omits required output/notices")
    selection = json.loads(SELECTION.read_text())
    if json.loads((output / "selection.json").read_text()) != selection:
        raise ValueError("packaged selection differs from the reviewed subset")
    provenance = json.loads((output / "provenance.json").read_text())
    if provenance["revision"] != selection["revision"] or provenance["archive_sha256"] != selection["archive_sha256"]:
        raise ValueError("candidate input pin differs")
    expected_ids = {f"{key}/{layer}" for key, visual in selection["visuals"].items()
                    for layer in ("layers", "open_layers") if layer in visual}
    if set(provenance["visuals"]) != expected_ids:
        raise ValueError("candidate coverage differs from the selected subset")
    for rows in provenance["visuals"].values():
        for row in rows:
            source = "sources/" + safe_path(row["source"])
            if source not in manifest or manifest[source] != row["source_sha256"]:
                raise ValueError(f"source pixel inventory differs: {source}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--source-dir", type=Path)
    parser.add_argument("--notices-dir", type=Path, help="defaults to --source-dir")
    parser.add_argument("--output", type=Path, default=OUTPUT)
    parser.add_argument("--check", action="store_true", help="compare regenerated outputs")
    parser.add_argument("--verify", action="store_true", help="check packaged files without upstream inputs")
    args = parser.parse_args()
    if args.verify:
        verify(args.output)
        print("Candidate files, coverage, provenance and notices verified.")
        return
    if args.archive is None or args.source_dir is None:
        parser.error("--archive and --source-dir are required for conversion")
    selection = json.loads(SELECTION.read_text())
    outputs = generate(args.archive.read_bytes(), selection, args.source_dir)
    # Notices are reviewed inputs, never invented from the archive (which omits them).
    for name in ("CC-BY-SA-3.0.txt", "upstream-readme.md", "upstream-LICENSE.txt", "CREDITS.md"):
        outputs[name] = ((args.notices_dir or args.source_dir) / name).read_bytes()
    outputs["checksums.json"] = (json.dumps({n: digest(b) for n, b in sorted(outputs.items())},
                                          indent=2) + "\n").encode()
    previous = args.output / "checksums.json"
    if not args.check and previous.is_file():
        for stale in set(json.loads(previous.read_text())) - set(outputs):
            path = (args.output / safe_path(stale)).resolve()
            if not path.is_relative_to(args.output.resolve()):
                raise ValueError(f"generated cleanup escapes output directory: {path}")
            path.unlink()
    for name, data in outputs.items():
        path = args.output / safe_path(name)
        if args.check:
            if not path.is_file() or path.read_bytes() != data:
                raise ValueError(f"stale generated candidate file: {path}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    verify(args.output)
    print(f"Candidate subset {'checked' if args.check else 'written'}: {args.output}")


if __name__ == "__main__":
    main()
