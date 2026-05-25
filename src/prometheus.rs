
pub enum MetricType {
    Gauge,
    #[allow(dead_code)]
    Counter,
}

pub struct PrometheusMetric {
    pub name: String,
    pub metric_type: MetricType,
    pub description: Option<String>, // will be used as HELP in Prometheus exposition format
    pub attributes: Vec<(String, String)>,
    pub value: u64,
}

pub struct PrometheusMetricCollection {
    pub metrics: Vec<PrometheusMetric>,
}

impl PrometheusMetric {
    pub fn new(name: &str, metric_type: MetricType, description: &str, attributes: Vec<(String, String)>, value: u64) -> PrometheusMetric {
        PrometheusMetric {
            name: name.to_string(),
            metric_type: metric_type,
            description: Some(description.to_string()),
            attributes: attributes,
            value: value,
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        if let Some(desc) = &self.description {
            result.push_str(&format!("# HELP {} {}\n", self.name, desc));
        }
        result.push_str(&format!("# TYPE {} {}\n", self.name, match self.metric_type {
            MetricType::Gauge => "gauge",
            MetricType::Counter => "counter",
        }));
        result.push_str(&format!("{}{{{}}} {}\n", self.name, Self::build_attributes(&self.attributes), self.value));
        // attributes will be rendered in render_sample method
        return result;
    }

    pub fn build_attributes(attributes: &Vec<(String, String)>) -> String {
        let mut result = String::new();
        for (key, value) in attributes {
            if !result.is_empty() {
                result.push_str(",");
            }
            result.push_str(&format!("{}=\"{}\"", key, value));
        }
        return result;
    }

}

impl PrometheusMetricCollection {
    pub fn new() -> Self {
        PrometheusMetricCollection {
            metrics: Vec::new(),
        }
    }

    pub fn add_metric(&mut self, metric: PrometheusMetric) {
        self.metrics.push(metric);
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for metric in &self.metrics {
            result.push_str(&metric.to_string());
        }
        return result;
    }
}
