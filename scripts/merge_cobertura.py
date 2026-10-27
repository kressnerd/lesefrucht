#!/usr/bin/env python3
"""Merge two Cobertura reports by summing per-line hits."""

from __future__ import annotations

import sys
import xml.etree.ElementTree as ET


def read(path: str) -> ET.ElementTree:
    return ET.parse(path)


def merge_lines(tree: ET.ElementTree) -> dict[tuple[str, int], int]:
    hits: dict[tuple[str, int], int] = {}
    for class_node in tree.iter("class"):
        filename = class_node.get("filename", "")
        for line in class_node.iter("line"):
            number = int(line.get("number", "0"))
            count = int(line.get("hits", "0"))
            key = (filename, number)
            hits[key] = hits.get(key, 0) + count
    return hits


def main() -> int:
    if len(sys.argv) != 4:
        print("usage: merge_cobertura.py <a.xml> <b.xml> <out.xml>", file=sys.stderr)
        return 2
    left, right, output = sys.argv[1:]
    hits = merge_lines(read(left))
    for key, count in merge_lines(read(right)).items():
        hits[key] = hits.get(key, 0) + count
    root = ET.Element("coverage")
    packages = ET.SubElement(root, "packages")
    package = ET.SubElement(packages, "package", name="lesefrucht")
    classes = ET.SubElement(package, "classes")
    by_file: dict[str, list[tuple[int, int]]] = {}
    for (filename, number), count in hits.items():
        by_file.setdefault(filename, []).append((number, count))
    for filename, lines in sorted(by_file.items()):
        class_node = ET.SubElement(classes, "class", filename=filename, name=filename)
        lines_node = ET.SubElement(class_node, "lines")
        for number, count in sorted(lines):
            ET.SubElement(lines_node, "line", number=str(number), hits=str(count))
    ET.ElementTree(root).write(output, encoding="utf-8", xml_declaration=True)
    print(f"merged {len(hits)} lines into {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
