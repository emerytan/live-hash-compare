# live-hash-compare

A Rust CLI tool to compare MD5 hashes generated from a given path against a reference md5 file, reporting any differences.

It can also be used as a standalone multi-threaded md5 checksum generator.

By default it will use 4 CPU cores, users should set a reasonable thread count based on current or anticipated CPU demand from other applications.

## Usage

```
Multithreaded md5 generator or compare an existing md5 file to a path in your filesystem.

Usage: live-hash-compare -f <PATH> (-m <MD5_FILE> | --generate) [-r <REPORT_PATH>] [--threads <NUM>]

Options:
  -f, --files-path <FILES_PATH>  
  -m, --md5-file <MD5_FILE>      
  -r, --report-path <PATH>       
  -g, --generate                 
  -t, --threads <NUM>            [default: 4]
  -h, --help                     Print help
  -V, --version                  Print version
```

## Examples   

### Compare existing md5
Generate md5 checksums and compare them to a reference md5 file
```
live-hash-compare --files-path ./data --md5-file ./reference.md5 --report-path ./report.txt
```
The report will list any files with mismatched hashes, missing files, or extra files in the reference.
If the report path option is not used the report will be written to the current working directory and formatted with basename of the `--files-path` argument and a timestamp.

### Generate new md5 file
Generate md5 file for given path, save it to the path, use 4 cpu cores (default)
```
live-hash-compare --generate --files-path ./data 
```
An md5 file will be written to the directory specified in the `--files-path|-f` option.  The formatting of the md5 file is `hashes_$(date +"%Y%m%d_%H%M%S).md5`.  If the `--report-path|-r` is given you must include a proper path and md5 file extension.


## -h, --help Output




