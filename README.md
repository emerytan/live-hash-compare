# live-hash-compare

A Rust CLI tool to compare MD5 hashes generated from a given path against a reference md5 file, reporting any differences.

It can also be used as a standalone multi-threaded md5 checksum generator.

By default it will use all CPU cores, set a reasonable thread count based on current or anticipated CPU demand from other applications. 

## Usage

```
live-hash-compare --files-path <DIR> --md5-file <MD5FILE> --report-path <REPORT>
```

- `--files-path`  Path to directory containing files to hash
- `--md5-file`    Path to md5 reference file (format: `<md5sum> <filename>` per line)
- `--report-path` Path to write the results report
- `--generate`    Create md5 file for path given in `--files-path`.  
- `--threads`    Set the number of CPU cores to be used.  Default is all.  

## Examples   

### Compare existing md5
Generate md5 checksums and compare them to a reference md5 file
```
live-hash-compare --files-path ./data --md5-file ./reference.md5 --report-path ./report.txt
```
The report will list any files with mismatched hashes, missing files, or extra files in the reference.     

### Generate new md5 file
Generate md5 file for given path, save it to the path, use 4 cpu cores
```
live-hash-compare --generate --files-path ./data 
```

## -h, --help Output

```
Usage: live-hash-compare [OPTIONS] --files-path <FILES_PATH> <--md5-file <MD5_FILE>|--generate>

Options:
  -f, --files-path <FILES_PATH>  
  -m, --md5-file <MD5_FILE>      
  -r, --report-path <PATH>       
      --generate                 
      --threads <NUM>            
  -h, --help                     Print help
  -V, --version                  Print version
```


