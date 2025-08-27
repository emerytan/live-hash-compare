# live-hash-compare

A Rust CLI tool to compare MD5 hashes generated from a given path against a reference md5 file, reporting any differences.

## Usage

```
live-hash-compare --files-path <DIR> --md5-file <MD5FILE> --report-path <REPORT>
```

- `--files-path`  Path to directory containing files to hash
- `--md5-file`    Path to md5 reference file (format: `<md5sum> <filename>` per line)
- `--report-path` Path to write the results report

## Example

```
live-hash-compare --files-path ./data --md5-file ./reference.md5 --report-path ./report.txt
```

## Output

The report will list any files with mismatched hashes, missing files, or extra files in the reference.
