# lesefrucht brand assets

The mark is a pear built from a stack of paper sheets: fruit made of pages. One
sheet, at the waist, is amber and runs out of the silhouette into a cream
citation bracket, so the icon reads as *a highlighted passage with its
citation*. No magnifying glass, no bookshelf, no robot.

## Two optical sizes, not one

A single artwork cannot serve 16 px and 512 px. At a target size *S*, a solid
feature needs at least 1.5 device pixels and a gap at least 1 px; on the 1024
unit canvas that is **1536/S** and **1024/S** units.

| Target | Minimum feature | Minimum gap |
|---|---|---|
| 16 px | 96 u | 64 u |
| 32 px | 48 u | 32 u |
| 64 px | 24 u | 16 u |
| 96 px | 16 u | 11 u |

The primary mark's sheets are 56 u with 14 u gaps, so it only resolves from
about 96 px upward. At 16 px the same geometry renders as one olive mass with a
floating amber dash:

| Element | Units | Device px at 16 |
|---|---|---|
| sheet | 56 | 0.88 |
| gap | 14 | 0.22 |
| stem | 28 | 0.44 |
| citation bracket, bar | 40 | 0.63 |
| citation bracket, tab | 20 | 0.31 |

That is why there are two files:

| File | Use at | Contains |
|---|---|---|
| `logo.svg` | 96–1024 px | sheet seams, amber passage, citation bracket, stem and leaf |
| `logo-small.svg` | 16–64 px | solid pear, amber band, leaf; no seams, no bracket |
| `logo-square.svg` | 180, 192, 512 px | full-bleed tile, mark at 85 %, for platforms that mask it themselves |

`logo-small.svg` gives up the page seams and the citation bracket because
neither can be resolved at that size. A solid pear with one amber band is the
smallest form that still reads as this brand.

## Files

| Path | Contents |
|---|---|
| `logo.svg` | primary mark, rounded tile |
| `logo-small.svg` | small optical size |
| `logo-square.svg` | full-bleed square tile, mark at 85 % |
| `png/icon-16.png` … `png/icon-64.png` | small optical size |
| `png/icon-96.png` … `png/icon-1024.png` | primary mark |
| `png/apple-touch-icon.png` | 180 px, square |
| `png/icon-192.png`, `png/icon-maskable-512.png` | square, masked by the platform |
| `favicon.ico` | 16, 32, 48 px |

## Palette

| Role | Hex |
|---|---|
| Deep ink green (tile) | `#1F3D2E` |
| Warm paper cream (pages, bracket, stem, leaf) | `#F3E9D2` |
| Muted amber (the highlighted passage only) | `#D9A441` |

Flat fills only. No text, no gradient, no shadow.

## Rebuilding the rasters

```sh
./build.sh
```

Needs ImageMagick with the librsvg delegate. Sources are the three SVGs in this
directory; the PNGs and `favicon.ico` are derived and can be deleted and
regenerated at any time.
