use r2r::visualization_msgs::msg::{Marker, MarkerArray};
use r2r::{Publisher, QosProfile, Timer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Represents the type of update to perform on a marker.
#[derive(Clone, Debug)]
pub enum UpdateType {
    Add,
    Modify,
    Delete,
    DeleteAll,
}

/// Holds information about a marker update.
#[derive(Clone, Debug)]
struct UpdateContext {
    pub update_type: UpdateType,
    pub marker: Marker,
}

/// A server that manages and publishes markers regularly.
#[derive(Clone)]
pub struct RegularMarkerServer {
    // pub topic_namespace: String,
    pub topic: String,
    marker_contexts: Arc<Mutex<HashMap<String, Marker>>>,
    pending_updates: Arc<Mutex<HashMap<String, UpdateContext>>>,
}

impl RegularMarkerServer {
    /// Creates a new `RegularMarkerServer`.
    ///
    /// # Arguments
    ///
    /// * `topic_namespace` - The namespace for the ROS topic.
    /// * `topic_name` - The name of the ROS topic.
    /// * `node` - A reference to the ROS node.
    pub fn new(topic: &str, node: &Arc<Mutex<r2r::Node>>) -> Self {
        let publisher_topic = format!("{}", topic);
        let mut publisher_qos = QosProfile::default();
        publisher_qos.depth = 100;

        // Create a publisher for MarkerArray messages.
        let publisher = node
            .lock()
            .unwrap()
            .create_publisher::<MarkerArray>(&publisher_topic, publisher_qos)
            .expect("Failed to create publisher");

        // Create a timer for periodic publishing.
        let timer = node
            .lock()
            .unwrap()
            .create_wall_timer(std::time::Duration::from_millis(20))
            .unwrap();

        let marker_contexts = Arc::new(Mutex::new(HashMap::new()));
        let pending_updates = Arc::new(Mutex::new(HashMap::new()));

        let marker_contexts_clone = marker_contexts.clone();

        // Spawn a task to publish markers periodically.
        tokio::task::spawn(async move {
            match Self::marker_array_publisher(marker_contexts_clone, publisher, timer).await {
                Ok(()) => (),
                Err(e) => r2r::log_error!("asdf", "Marker array publisher failed with: '{}'.", e),
            };
        });

        let marker_contexts_clone = marker_contexts.clone();
        let pending_updates_clone = pending_updates.clone();

        Self {
            // topic_namespace: topic_namespace.to_string(),
            topic: topic.to_string(),
            marker_contexts: marker_contexts_clone,
            pending_updates: pending_updates_clone,
        }
    }

    /// Inserts a new marker.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique name of the marker.
    /// * `marker` - The marker data to insert.
    pub fn insert(&self, name: &str, marker: Marker) {
        let mut pending_updates = self.pending_updates.lock().unwrap();

        // Add or update the pending update for the marker.
        let update_context =
            pending_updates
                .entry(name.to_string())
                .or_insert_with(|| UpdateContext {
                    update_type: UpdateType::Add,
                    marker: marker.clone(),
                });

        update_context.update_type = UpdateType::Add;
        update_context.marker = marker;

        println!("Marker added with name '{}'", name);
    }

    /// Deletes a marker by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique name of the marker to delete.
    pub fn delete(&self, name: &str) {
        let marker_contexts = self.marker_contexts.lock().unwrap();
        let mut pending_updates = self.pending_updates.lock().unwrap();

        if let Some(marker_context) = marker_contexts.get(name) {
            pending_updates.insert(
                name.to_string(),
                UpdateContext {
                    update_type: UpdateType::Delete,
                    marker: marker_context.clone(),
                },
            );
        }
    }

    /// Applies pending updates to markers.
    pub fn apply_changes(&self) {
        let mut marker_contexts = self.marker_contexts.lock().unwrap();
        let mut pending_updates = self.pending_updates.lock().unwrap();

        if pending_updates.is_empty() {
            println!("No changes to apply");
            return;
        }

        for (name, update_context) in pending_updates.iter() {
            match update_context.update_type {
                UpdateType::Add => {
                    marker_contexts.entry(name.clone()).or_insert_with(|| {
                        let mut marker_context = update_context.marker.clone();
                        marker_context.action = Marker::ADD as i32;
                        marker_context
                    });
                }
                UpdateType::Modify => {
                    if let Some(marker_context) = marker_contexts.get_mut(name) {
                        marker_context.pose = update_context.marker.pose.clone();
                        marker_context.header = update_context.marker.header.clone();
                        marker_context.action = Marker::MODIFY as i32;
                    } else {
                        println!("Pending modify update for non-existing marker '{}'.", name);
                    }
                }
                UpdateType::Delete => {
                    if let Some(marker_context) = marker_contexts.get_mut(name) {
                        marker_context.action = Marker::DELETE as i32;
                    } else {
                        println!("Pending delete update for non-existing marker '{}'.", name);
                    }
                }
                UpdateType::DeleteAll => {
                    for marker_context in marker_contexts.values_mut() {
                        marker_context.action = Marker::DELETEALL as i32;
                    }
                }
            }
        }

        pending_updates.clear();
    }

    /// Publishes marker arrays periodically.
    ///
    /// # Arguments
    ///
    /// * `marker_contexts` - Shared marker contexts.
    /// * `publisher` - The publisher to publish marker arrays.
    /// * `timer` - Timer for periodic publishing.
    async fn marker_array_publisher(
        marker_contexts: Arc<Mutex<HashMap<String, Marker>>>,
        publisher: Publisher<MarkerArray>,
        mut timer: Timer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut markers = marker_contexts.lock().unwrap().clone();
            let mut update_msg = MarkerArray::default();

            // Collect markers to publish.
            for (_, marker) in &markers {
                update_msg.markers.push(marker.clone());
            }

            // Publish the marker array.
            publisher
                .publish(&update_msg)
                .expect("Failed to publish update");

            // Update marker contexts based on actions.
            for (name, marker) in markers.clone().iter() {
                match marker.action {
                    2 => {
                        // Remove markers marked for deletion.
                        let _ = markers.remove(name);
                    }
                    3 => {
                        // Clear all markers if delete all action is set.
                        markers.clear();
                    }
                    _ => (),
                }
            }

            *marker_contexts.lock().unwrap() = markers;

            timer.tick().await?;
        }
    }
}
