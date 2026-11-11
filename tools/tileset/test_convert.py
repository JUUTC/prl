"""Synthetic conversion fixtures and committed package validation; no network."""

import io
import json
from pathlib import Path
import shutil
import tempfile
import unittest
import zipfile

from PIL import Image

import convert


def archive(entries=None, offset=0):
    config = {
        "tile_info": [{"width": 32, "height": 32}],
        "tiles-new": [
            {"file": "a.png", "tiles": [{"id": ["alias", "base"], "fg": 0}]},
            {"file": "b.png", "sprite_height": 64, "sprite_offset_y": offset,
             "tiles": entries or [{"id": "tall", "fg": [{"weight": 2, "sprite": 1}]}]},
        ],
    }
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w") as z:
        z.writestr(convert.PREFIX + "tile_config.json", json.dumps(config))
        for name, size, color in [("a.png", (32, 32), "red"), ("b.png", (32, 64), "blue")]:
            image = io.BytesIO()
            Image.new("RGBA", size, color).save(image, format="PNG")
            z.writestr(convert.PREFIX + name, image.getvalue())
    return buffer.getvalue()


class ConversionTests(unittest.TestCase):
    def test_notice_hashes_survive_git_line_ending_normalization(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "LICENSE.txt"
            path.write_bytes(b"Licence\r\nTerms\r\n")
            windows = convert.notice_bytes(path)
            path.write_bytes(b"Licence\nTerms\n")
            self.assertEqual(windows, convert.notice_bytes(path))

    def test_hash_paths_and_invalid_indices_fail_explicitly(self):
        data = archive()
        with self.assertRaisesRegex(ValueError, "SHA-256"):
            convert.Tileset(data, "bad")
        for name in ("../escape.png", "/abs.png", "C:/escape.png", r"a\b.png"):
            with self.assertRaisesRegex(ValueError, "unsafe"):
                convert.safe_path(name)
        for value in (-1, 200, True):
            data = archive([{"id": "bad", "fg": value}])
            with self.assertRaisesRegex(ValueError, "index"):
                convert.Tileset(data, convert.digest(data)).layer("bad", "fg", None, 0)

    def test_aliases_global_sheet_boundaries_and_offsets(self):
        data = archive(offset=-32)
        tiles = convert.Tileset(data, convert.digest(data))
        image, offset, record = tiles.layer("alias", "fg", None, 0)
        self.assertEqual(image.getpixel((0, 0)), (255, 0, 0, 255))
        image, offset, record = tiles.layer("tall", "fg", None, 0)
        self.assertEqual((image.size, offset, record["index"]), ((32, 64), (0, -32), 1))

    def test_direction_arrays_animation_and_ambiguous_ids_are_not_silently_flattened(self):
        for entries in ([{"id": "bad", "fg": [0, 1]}],
                        [{"id": "bad", "fg": 0, "animated": True}],
                        [{"id": "bad", "fg": 0}, {"id": "bad", "fg": 1}]):
            data = archive(entries)
            with self.assertRaises(ValueError):
                convert.Tileset(data, convert.digest(data)).layer("bad", "fg", None, 0)

    def test_source_mismatch_and_clipping_reject_without_output(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)
            Image.new("RGBA", (32, 64), "blue").save(source / "source.png")
            selection = {"canvas": [32, 64], "origin": [0, -32]}
            layers = [["tall", "fg", None, 0, "source.png"]]
            data = archive(offset=-32)
            tiles = convert.Tileset(data, convert.digest(data))
            first = convert.composite(tiles, layers, selection, source)
            second = convert.composite(tiles, layers, selection, source)
            self.assertEqual(first[0].tobytes(), second[0].tobytes())
            self.assertEqual(first[1], second[1])
            selection["origin"] = [0, 0]
            with self.assertRaisesRegex(ValueError, "clipped"):
                convert.composite(tiles, layers, selection, source)
            Image.new("RGBA", (32, 64), "red").save(source / "source.png")
            with self.assertRaisesRegex(ValueError, "audited source"):
                convert.composite(tiles, layers, selection, source)

    def test_composition_preserves_layer_order_and_reproduces_export_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)
            for name, size, color in [("base.png", (32, 32), "red"),
                                      ("tall.png", (32, 64), "blue")]:
                Image.new("RGBA", size, color).save(source / name)
            (source / "history.json").write_text(json.dumps({"base.png": [], "tall.png": []}))
            data = archive(offset=-32)
            selection = {
                "release": "test", "revision": "test", "archive_sha256": convert.digest(data),
                "canvas": [32, 64], "origin": [0, -32], "scale": 2,
                "visuals": {"prl:player": {
                    "kind": "actor", "layers": [
                        ["tall", "fg", None, 0, "tall.png"],
                        ["base", "fg", None, 0, "base.png"],
                    ],
                }},
            }
            outputs = convert.generate(data, selection, source)
            self.assertEqual(outputs, convert.generate(data, selection, source))
            image = Image.open(io.BytesIO(outputs["ultica.png"]))
            self.assertEqual(image.size, (34, 66))
            self.assertEqual(image.getpixel((1, 1)), (0, 0, 255, 255))
            self.assertEqual(image.getpixel((1, 33)), (255, 0, 0, 255))
            self.assertEqual(image.getpixel((0, 0)), (0, 0, 255, 255))
            self.assertEqual(image.getpixel((33, 65)), (255, 0, 0, 255))
            manifest = outputs["ultica.ron"].decode()
            self.assertIn('status: "candidate"', manifest)
            self.assertIn("scale: 2.0", manifest)
            self.assertIn("offset_x: 0.0, offset_y: 32.0", manifest)
            self.assertIn("left: None", manifest)

    def test_padding_extrudes_each_edge_without_changing_sprite_pixels(self):
        data = archive(offset=-32)
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)
            Image.new("RGBA", (32, 32), "red").save(source / "red.png")
            Image.new("RGBA", (32, 64), "blue").save(source / "blue.png")
            (source / "history.json").write_text(json.dumps({"red.png": [], "blue.png": []}))
            selection = {
                "release": "test", "revision": "test", "archive_sha256": convert.digest(data),
                "canvas": [32, 64], "origin": [0, -32], "scale": 2,
                "visuals": {
                    "prl:floor": {"kind": "tile", "layers": [["base", "fg", None, 0, "red.png"]]},
                    "prl:player": {"kind": "actor", "layers": [["tall", "fg", None, 0, "blue.png"]]},
                },
            }
            outputs = convert.generate(data, selection, source)
            atlas = Image.open(io.BytesIO(outputs["ultica.png"]))
            self.assertEqual(atlas.size, (68, 66))
            tileset = convert.Tileset(data, convert.digest(data))
            for index in range(2):
                x = index * 34 + 1
                frame = atlas.crop((x, 1, x + 32, 65))
                visual = list(sorted(selection["visuals"]))[index]
                expected, _ = convert.composite(
                    tileset, selection["visuals"][visual]["layers"], selection, source)
                self.assertEqual(frame.tobytes(), expected.tobytes())
                for y in range(64):
                    self.assertEqual(atlas.getpixel((x - 1, y + 1)), frame.getpixel((0, y)))
                    self.assertEqual(atlas.getpixel((x + 32, y + 1)), frame.getpixel((31, y)))
                    # A fractional camera can move the sample just outside the
                    # frame. Nearest sampling must still see that frame's edge.
                    for sample_x in (-0.25, 0.25, 31.75, 32.25):
                        atlas_x = x + int(sample_x // 1)
                        edge_x = max(0, min(31, int(sample_x // 1)))
                        self.assertEqual(atlas.getpixel((atlas_x, y + 1)),
                                         frame.getpixel((edge_x, y)))
                for column in range(32):
                    self.assertEqual(atlas.getpixel((x + column, 0)), frame.getpixel((column, 0)))
                    self.assertEqual(atlas.getpixel((x + column, 65)), frame.getpixel((column, 63)))
                for dx, sx in ((-1, 0), (32, 31)):
                    for dy, sy in ((0, 0), (65, 63)):
                        self.assertEqual(atlas.getpixel((x + dx, dy)), frame.getpixel((sx, sy)))

    def test_package_verification_rejects_missing_notices_or_untracked_art(self):
        output = convert.OUTPUT
        convert.verify(output)
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory)
            shutil.copytree(output, target, dirs_exist_ok=True)
            (target / "untracked.png").write_bytes(b"not reviewed")
            with self.assertRaisesRegex(ValueError, "untracked"):
                convert.verify(target)
            (target / "untracked.png").unlink()
            (target / "CC-BY-SA-3.0.txt").unlink()
            with self.assertRaisesRegex(ValueError, "missing"):
                convert.verify(target)


if __name__ == "__main__":
    unittest.main()
