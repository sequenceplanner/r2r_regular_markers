use r2r::visualization_msgs::msg::{Marker, MarkerArray};
use r2r::{Publisher, QosProfile};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct RegularMarkerServer {
    pub topic_namespace: String,
    pub topic_name: String,
    pub markers: Arc<Mutex<HashMap<String, Marker>>>,
    pub publisher: Publisher<MarkerArray>,
}

impl RegularMarkerServer {
    pub fn new(topic_namespace: &str, topic_name: &str, node: Arc<Mutex<r2r::Node>>) -> Self {
        let publisher_topic = format!("{}/{}", topic_namespace, topic_name);
        let mut publisher_qos = QosProfile::default();
        publisher_qos.depth = 100;

        let publisher = node
            .lock()
            .unwrap()
            .create_publisher::<MarkerArray>(&publisher_topic, publisher_qos)
            .expect("Failed to create publisher");

        let markers = Arc::new(Mutex::new(HashMap::new()));
        let markers_clone = markers.clone();

        // Should probably only publish when there are some changes, test with moving tf frames...
        // Make and example with some rotating tf frame.

        Self {
            topic_namespace: topic_namespace.to_string(),
            topic_name: topic_name.to_string(),
            markers: markers_clone,
            publisher,
        }
    }
}
