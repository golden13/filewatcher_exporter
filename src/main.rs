extern crate prometheus_exporter_base;
extern crate clap;
extern crate log;
extern crate env_logger;
extern crate sysinfo;
extern crate gethostname;

use clap::{crate_authors, crate_name, crate_version, Arg};
use log::{info, trace};
use prometheus_exporter_base::{render_prometheus, MetricType, PrometheusMetric};
use std::env;
use std::fs::metadata;
use std::net::{SocketAddr};
use std::process;
use std::time::SystemTime;
use sysinfo::{SystemExt, DiskExt};
use sysinfo::DiskType;
use std::str::from_utf8;



#[derive(Debug, Clone, Default)]
struct MyOptions<'a> {
    targets:Vec<&'a str>
}


fn disk_type_to_str(dtype: &DiskType) -> &str {
    match *dtype {
        DiskType::HDD => "HDD",
        DiskType::SSD => "SSD",
        DiskType::Unknown(Any) => "Unknown"
    }
}

fn get_last_updated(str: &str) -> (u64, u64) {
    let metadata = metadata(str).unwrap();
    let len = metadata.len();
    let time = metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    return (time, len);
}

fn main() {
    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("port")
                .short("p")
                .help("exporter port")
                .default_value("9104")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .help("verbose logging")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .help("hostname")
                .default_value("0.0.0.0")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("targets")
                .short("t")
                .help("list of target files to monitor")
                .default_value("test.file;test2.file") // debug
                .takes_value(true)
        )
        .get_matches();

    if !matches.is_present("targets") {
        println!("Error: no `target` specified. Please, run filewatcher_exporter -t [list_of_files_separated_by_;]");
        process::exit(1);
    }

    if matches.is_present("verbose") {
        env::set_var(
            "RUST_LOG",
            format!("folder_size=trace,{}=trace", crate_name!()),
        );
    } else {
        env::set_var(
            "RUST_LOG",
            format!("folder_size=info,{}=info", crate_name!()),
        );
    }
    env_logger::init();

    info!("using matches: {:?}", matches);

    let target_list = matches.value_of("targets").unwrap();

    let bind = matches.value_of("port").unwrap();
    let bind = u16::from_str_radix(&bind, 10).expect("Error: Port must be a valid number");
    let host =  format!("{}:{}", matches.value_of("host").unwrap(), bind);

    let options: Vec<String> = target_list.split(";").map(|s| s.to_string()).collect();

    let addr: SocketAddr = host
        .parse()
        .expect("Error: Unable to parse socket address");

    info!("Starting exporter on {} ...", addr);

    render_prometheus(addr, options, |request, options| {
        async move {
            // to satisfy the borrow checker? Or exclude memory leaks
            let options = options.clone();
            let hostname = gethostname::gethostname();

            trace!(
                "Request: {:?}, Options: {:?})",
                request,
                options
            );

            let filewatcher_file_modified = PrometheusMetric::new("filewatcher_file_modified", MetricType::Gauge, "The timestamp when the file was last modified");
            let filewatcher_file_size = PrometheusMetric::new("filewatcher_file_size", MetricType::Gauge, "The size of the file in bytes");

            let mut s = filewatcher_file_modified.render_header();
            let mut s2 = filewatcher_file_size.render_header();

            for elem in options.iter() {
                let s_slice: &str = &elem[..];
                //println!("Reading file: {:?}", s_slice);
                let (modified, len) = get_last_updated(s_slice);
                let mut attributes = Vec::new();
                attributes.push(("filename", s_slice));
                attributes.push(("host", hostname.to_str().unwrap()));

                s.push_str(&filewatcher_file_modified.render_sample(Some(&attributes), modified));
                s2.push_str(&filewatcher_file_size.render_sample(Some(&attributes), len));
            }

            s.push_str(&s2);

            // Get system info
            //let system_info = get_system_info();
            let mut system = sysinfo::System::new();
            system.refresh_all();
            let mut attributes = Vec::new();
            attributes.push(("host", hostname.to_str().unwrap()));

            // mem_swap_total
            // TODO: add host attribute?
            let pmetric_mem_swap_total = PrometheusMetric::new("mem_swap_total", MetricType::Gauge, "mem_swap_total collected metric");
            s.push_str(&pmetric_mem_swap_total.render_header());
            s.push_str(&pmetric_mem_swap_total.render_sample(Some(&attributes), system.get_total_swap()));

            // mem_total
            // TODO: add host attribute?
            let pmetric_mem_total = PrometheusMetric::new("mem_total", MetricType::Gauge, "mem_total collected metric");
            s.push_str(&pmetric_mem_total.render_header());
            s.push_str(&pmetric_mem_total.render_sample(Some(&attributes), system.get_total_memory()));

            // mem_used
            // TODO: add host attribute?
            let pmetric_mem_used = PrometheusMetric::new("mem_used", MetricType::Gauge, "mem_used collected metric");
            s.push_str(&pmetric_mem_used.render_header());
            s.push_str(&pmetric_mem_used.render_sample(Some(&attributes), system.get_used_memory()));

            // mem_swap_used
            // INFO: Telegraf doesn't have this metric, only mem_swap_free.
            let pmetric_mem_swap_used = PrometheusMetric::new("mem_swap_used", MetricType::Gauge, "mem_swap_used collected metric");
            s.push_str(&pmetric_mem_swap_used.render_header());
            s.push_str(&pmetric_mem_swap_used.render_sample(Some(&attributes), system.get_used_swap()));

            // Disks information
            let pmetric_disk_free = PrometheusMetric::new("disk_free", MetricType::Gauge, "disk_free collected metric");
            s.push_str(&pmetric_disk_free.render_header());

            let pmetric_disk_total = PrometheusMetric::new("disk_total", MetricType::Gauge, "disk_total collected metric");
            let mut s2 = pmetric_disk_total.render_header();

            for disk in system.get_disks() {
                let mut attributes2 = Vec::new();
                let path = disk.get_name().to_str().unwrap();
                attributes2.push(("device", path));

                attributes2.push(("host", hostname.to_str().unwrap()));

                let fstype = disk.get_file_system();
                attributes2.push(("fstype", from_utf8(fstype).unwrap()));

                attributes2.push(("path", disk.get_mount_point().to_str().unwrap()));

                let dtype_enum = &disk.get_type();
                let dtype = disk_type_to_str(dtype_enum);

                attributes2.push(("type", dtype));

                // TODO: mode="rw", host="xxx"

                s.push_str(&pmetric_disk_free.render_sample(Some(&attributes2), disk.get_available_space()));
                s2.push_str(&pmetric_disk_total.render_sample(Some(&attributes2), disk.get_total_space()));
            }

            s.push_str(&s2);


            Ok(s)
        }
    });
}