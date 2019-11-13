# Prometheus Filewatcher Exporter
File meta information exporter written in Rust.
Provides a few file and system metrics for [Prometheus](https://prometheus.io/)

| Metric name | Description | 
|---------------------------|---------------|
| filewatcher_file_modified | File last modified timestamp |
| filewatcher_file_size | File size in bytes |
| mem_total | Total amount of server memory in bytes |
| mem_used | Total used memory in bytes |
| mem_swap_total | Swap size in bytes |
| mem_swap_used | Used swap size in bytes |
| disk_free | Free disk space in bytes (for each mount point in system) |
| disk_total | Total dis size in bytes (for each mount point in system) |


# Run
```filewatcher_exporter -p 9104 -h 127.0.0.1 -t "/var/log/nginx/error.log;/var/log/nginx/access.log" ```

# Command line argments
```cmd
-p [port] - port number, default 9104
-h [host] - hostname, default 0.0.0.0
-v - verbouse 
-t [targets] - list of files to watch
```

# Compile binaries 
*NOTE:* Use nightly version:

```cargo +nightly build --release```

# Output example
```promql
# HELP filewatcher_file_modified The timestamp when the file was last modified
# TYPE filewatcher_file_modified gauge
filewatcher_file_modified{filename="test.file",host="golden"} 1572918373
filewatcher_file_modified{filename="test2.file",host="golden"} 1573664023
# HELP filewatcher_file_size The size of the file in bytes
# TYPE filewatcher_file_size gauge
filewatcher_file_size{filename="test.file",host="golden"} 7778
filewatcher_file_size{filename="test2.file",host="golden"} 7789
# HELP mem_swap_total mem_swap_total collected metric
# TYPE mem_swap_total gauge
mem_swap_total{host="golden"} 16658428
# HELP mem_total mem_total collected metric
# TYPE mem_total gauge
mem_total{host="golden"} 16304852
# HELP mem_used mem_used collected metric
# TYPE mem_used gauge
mem_used{host="golden"} 15128092
# HELP mem_swap_used mem_swap_used collected metric
# TYPE mem_swap_used gauge
mem_swap_used{host="golden"} 5595344
# HELP disk_free disk_free collected metric
# TYPE disk_free gauge
disk_free{device="sda2",host="golden",fstype="ext4",path="/",type="SSD"} 70964506624
disk_free{device="sda1",host="golden",fstype="vfat",path="/boot/efi",type="SSD"} 498417664
# HELP disk_total disk_total collected metric
# TYPE disk_total gauge
disk_total{device="sda2",host="golden",fstype="ext4",path="/",type="SSD"} 234587672576
disk_total{device="sda1",host="golden",fstype="vfat",path="/boot/efi",type="SSD"} 535805952
```

