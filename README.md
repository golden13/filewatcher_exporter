# Prometheus Filewatcher Exporter

This exporter watches files, providing the last modified date, file size, and a few system metrics for [Prometheus](https://prometheus.io/).

*Written in [Rust](https://github.com/rust-lang/rust)*

## What's New in Version 2.0.0
- Removed unnecessary dependencies
- Updated all external crates to their latest versions
- Migrated `filewatcher` to use async Tokio and Axum
- Added support for monitoring directories
- Completely rewritten codebase


> [!WARNING] 
> **The new version includes some breaking changes:**
> - The 'host' command-line parameter is now `-H` (instead of `-h`) due to a conflict with the help parameter.
> - The disk type attribute is now lowercase to keep it consistent with other metrics.

## Metrics

| Metric name | Description | Attributes |
|---------------------------|---------------|---------------|
| filewatcher_file_modified | File last modified timestamp | host, filename, filetype=[file \| directory \| symlink \| unknown] |
| filewatcher_file_size | File size in bytes | host, filename, filetype=[file \| directory \| symlink \| unknown] |
| mem_total | Total amount of server memory in bytes | host |
| mem_used | Total used memory in bytes | host |
| mem_swap_total | Swap size in bytes | host |
| mem_swap_used | Used swap size in bytes | host |
| disk_free | Free disk space in bytes (for each mount point in the system) | host, device=device name in the system, fstype=filesystem type (ext4, vfat, etc.), path=mount path, type=[hdd \| ssd \| unknown] |
| disk_total | Total disk size in bytes (for each mount point in the system) | host, device=device name in the system, fstype=filesystem type (ext4, vfat, etc.), path=mount path, type=[hdd \| ssd \| unknown] |

## Running the Exporter

```bash
filewatcher_exporter -p 9104 -H 127.0.0.1 -t "/var/log/nginx/error.log;/var/log/nginx/access.log"
```

With log messages in the output:
```bash 
filewatcher_exporter -p 9104 -H 127.0.0.1 -vv -t "/tmp/testfile.txt;/tmp/testfolder/test1.t" 2>&1
```

The target list can contain folders. They will also be monitored for their size and modification dates:
```bash
filewatcher_exporter -p 9104 -H 127.0.0.1 -vv -t "/tmp/testfolder;/tmp/log.log" 2>&1
```

## Command-line arguments
```cmd
-p [port]    - Port number (default: 9104)
-H [host]    - Hostname (default: 0.0.0.0)
-v, -vv, -vvv- Different verbosity levels 
-t [targets] - List of files/folders to watch, separated by ';'
```

## Compiling from Source 
```bash
cargo build --release
```

## Output example
```promql
# HELP filewatcher_file_modified The timestamp when the file was last modified
# TYPE filewatcher_file_modified gauge
filewatcher_file_modified{host="golden-cave",filename="/tmp/testfile.txt",filetype="file"} 1779723719
# HELP filewatcher_file_size The size of the file in bytes
# TYPE filewatcher_file_size gauge
filewatcher_file_size{host="golden-cave",filename="/tmp/testfile.txt",filetype="file"} 0
# HELP filewatcher_file_modified The timestamp when the file was last modified
# TYPE filewatcher_file_modified gauge
filewatcher_file_modified{host="golden-cave",filename="/tmp/testfolder",filetype="directory"} 1779730951
# HELP filewatcher_file_size The size of the file in bytes
# TYPE filewatcher_file_size gauge
filewatcher_file_size{host="golden-cave",filename="/tmp/testfolder",filetype="directory"} 4096
# HELP mem_swap_total mem_swap_total collected metric
# TYPE mem_swap_total gauge
mem_swap_total{host="golden-cave"} 8388604
# HELP mem_total mem_total collected metric
# TYPE mem_total gauge
mem_total{host="golden-cave"} 32126448
# HELP mem_used mem_used collected metric
# TYPE mem_used gauge
mem_used{host="golden-cave"} 20657420
# HELP mem_swap_used mem_swap_used collected metric
# TYPE mem_swap_used gauge
mem_swap_used{host="golden-cave"} 4957528
# HELP disk_free disk_free collected metric
# TYPE disk_free gauge
disk_free{host="golden-cave",device="nvme0n1p2",fstype="ext4",path="/",type="Unknown"} 57975042048
# HELP disk_total disk_total collected metric
# TYPE disk_total gauge
disk_total{host="golden-cave",device="nvme0n1p2",fstype="ext4",path="/",type="Unknown"} 982240026624
# HELP disk_free disk_free collected metric
# TYPE disk_free gauge
disk_free{host="golden-cave",device="nvme0n1p1",fstype="vfat",path="/boot/efi",type="Unknown"} 1118560256
# HELP disk_total disk_total collected metric
# TYPE disk_total gauge
disk_total{host="golden-cave",device="nvme0n1p1",fstype="vfat",path="/boot/efi",type="Unknown"} 1124999168
# HELP disk_free disk_free collected metric
# TYPE disk_free gauge
disk_free{host="golden-cave",device="sda2",fstype="ext4",path="/media/golden/f7959a96-285b-465f-a922-b510f50d1fbd",type="HDD"} 4912762949632
# HELP disk_total disk_total collected metric
# TYPE disk_total gauge
disk_total{host="golden-cave",device="sda2",fstype="ext4",path="/media/golden/f7959a96-285b-465f-a922-b510f50d1fbd",type="HDD"} 7936183619584
```