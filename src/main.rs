extern crate log;
extern crate env_logger;
extern crate sysinfo;
extern crate gethostname;

mod prometheus;
mod command_line;

use clap::Parser;
use log::{info};
use std::fs::metadata;
use std::time::SystemTime;
use sysinfo::{SystemExt, DiskExt};
use sysinfo::DiskType;
use std::str::from_utf8;
use axum::{Router, routing::get, extract::State};

fn disk_type_to_str(dtype: &DiskType) -> &str {
    match *dtype {
        DiskType::HDD => "hdd",
        DiskType::SSD => "ssd",
        DiskType::Unknown(_any) => "unknown"
    }
}

fn file_type_to_str(file_type: &std::fs::FileType) -> &str {
    if file_type.is_file() {
        "file"
    } else if file_type.is_dir() {
        "directory"
    } else if file_type.is_symlink() {
        "symlink"
    } else {
        "unknown"
    }
}

fn get_last_updated(str: &str) -> Result<(u64, u64, std::fs::FileType), Box<dyn std::error::Error>> {
    let metadata = metadata(str)?;
    let file_type = metadata.file_type();
    let len = metadata.len();
    let time = metadata.modified()?.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    Ok((time, len, file_type))
}

#[tokio::main]
async fn main() {
    let args = command_line::Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbosity.log_level_filter())
        .format_timestamp_millis()
        .format_module_path(false)
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "[{}] {}", record.level(), record.args())
        })
        .init();

    info!("Starting file watcher exporter on {}:{}", args.host, args.port);

    let target_list = args.targets;
    let options: Vec<String> = target_list.split(";").map(|s: &str| s.trim().to_string()).collect();
    info!("Target files: {}", options.join(", "));

    // check if files exist
    for target in options.iter() {
        if !std::path::Path::new(target).exists() {
            eprintln!("Error: file '{}' does not exist. Please, check the target list.", target);
            std::process::exit(1);
        }
    }

    // general attributes for all metrics, e.g. host="xxx"
    let mut main_props: Vec<(String, String)> = Vec::new();
    let hostname = gethostname::gethostname();
    main_props.push(("host".to_string(), hostname.to_str().unwrap().to_string()));

    let app = Router::new().route("/", get(process_request).with_state((options, main_props)));

    let full_addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&full_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/**
 * collect data and build a PrometheusMetricCollection
 */
async fn collect_data<'a>(options: Vec<String>, global_attributes: Vec<(String, String)>) -> prometheus::PrometheusMetricCollection {
    let mut result: prometheus::PrometheusMetricCollection = prometheus::PrometheusMetricCollection::new();
    let mut attributes: Vec<(String, String)> = Vec::new();
    attributes.extend(global_attributes);

    for target in options.iter() {
        match get_last_updated(target) {
            Ok((time, len, file_type)) => {
                let mut attributes2: Vec<(String, String)> = Vec::new();
                attributes2.extend(attributes.clone());
                attributes2.push(("filename".to_string(), target.to_string()));
                attributes2.push(("filetype".to_string(), file_type_to_str(&file_type).to_string()));

                result.add_metric(
                    prometheus::PrometheusMetric::new(
                        "filewatcher_file_modified", 
                        prometheus::MetricType::Gauge, 
                        "The timestamp when the file was last modified", 
                        attributes2.clone(),
                        time
                    )
                );

                result.add_metric(
                    prometheus::PrometheusMetric::new(
                        "filewatcher_file_size", 
                        prometheus::MetricType::Gauge, 
                        "The size of the file in bytes", 
                        attributes2.clone(),
                        len
                    )
                );
            }
            Err(e) => {
                eprintln!("Error reading file {}: {}", target, e);
            }
        }
    }

    // collect system metrics
    collect_system_metrics(&mut result, attributes.clone()).await;

    return result;
}

async fn collect_system_metrics(result: &mut prometheus::PrometheusMetricCollection, attributes: Vec<(String, String)>) {
    let mut system = sysinfo::System::new();
    system.refresh_all();
    
    result.add_metric(
        prometheus::PrometheusMetric::new(
            "mem_swap_total", 
            prometheus::MetricType::Gauge, 
            "mem_swap_total collected metric", 
            attributes.clone(),
            system.get_total_swap()
        )
    );

    result.add_metric(
        prometheus::PrometheusMetric::new(
            "mem_total", 
            prometheus::MetricType::Gauge, 
            "mem_total collected metric", 
            attributes.clone(),
            system.get_total_memory()
        )
    );

    result.add_metric(
        prometheus::PrometheusMetric::new(
            "mem_used", 
            prometheus::MetricType::Gauge, 
            "mem_used collected metric", 
            attributes.clone(),
            system.get_used_memory()
        )
    );

    result.add_metric(
        prometheus::PrometheusMetric::new(
            "mem_swap_used", 
            prometheus::MetricType::Gauge, 
            "mem_swap_used collected metric", 
            attributes.clone(),
            system.get_used_swap()
        )
    );

    for disk in system.get_disks() {
        let mut attributes2 = Vec::new();
        attributes2.extend(attributes.clone());
        
        let path = disk.get_name().to_str().unwrap();
        attributes2.push(("device".to_string(), path.to_string()));

        let fstype = disk.get_file_system();
        attributes2.push(("fstype".to_string(), from_utf8(fstype).unwrap().to_string()));
        attributes2.push(("path".to_string(), disk.get_mount_point().to_str().unwrap().to_string()));

        let dtype_enum = &disk.get_type();
        let dtype = disk_type_to_str(dtype_enum);

        attributes2.push(("type".to_string(), dtype.to_string()));

        // TODO: mode="rw", host="xxx"
        result.add_metric(
            prometheus::PrometheusMetric::new(
                "disk_free", 
                prometheus::MetricType::Gauge, 
                "disk_free collected metric", 
                attributes2.clone(),
                disk.get_available_space()
            )
        );

        result.add_metric(
            prometheus::PrometheusMetric::new(
                "disk_total", 
                prometheus::MetricType::Gauge, 
                "disk_total collected metric", 
                attributes2.clone(),
                disk.get_total_space()
            )
        );
    }
}

async fn process_request(State((options, main_props)): State<(Vec<String>, Vec<(String, String)>)>) -> String {
    let mut result: String = String::new();
    let mut global_attributes: Vec<(String, String)> = Vec::new();
    global_attributes.extend(main_props);

    let prometheus_metrics = collect_data(options, global_attributes).await;

    result.push_str(&prometheus_metrics.to_string());

    return result;
}
