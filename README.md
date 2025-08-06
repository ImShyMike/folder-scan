# folder-scan

Simple, lightweigth and blazingly fast folder scanner with a tree-like visualization that can be used to find space hogs.

![Usage demo](./assets/example.png)

---

## Usage

The program can be lanched with a path argument to automatically scan that folder.

```sh
$ folder-scan
or
$ folder-scan /abc/foo
```

## Optimizations

Rust with FLTK was the chosen tech stack as it has a very light memory footprint and amazing speed.

The program leverages cache-friendly data structures to store the scanned folder tree, fan-out/fan-in concurrency alongside a message passing concurrency model to handle multithreading for faster speeds at a low memory cost and threshold-based optimization while scanning to prune and avoid scanning very small folders.

## Benchmarks

> All tests were done using Arch Linux (with an SSD) and may differ on Windows (as some functions change behaviour when used on windows)

The idle memory usage on startup is about 11 MB.

The program was able to scan a 178 GB folder in around 7 seconds (4 for a rescan) while using only ~33 MB of RAM.
