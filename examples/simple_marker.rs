use r2r::geometry_msgs::msg::{Point, Pose, Quaternion, Transform, TransformStamped, Vector3};
use r2r::std_msgs::msg::Header;
use r2r::tf2_msgs::msg::TFMessage;
use r2r::visualization_msgs::msg::Marker;
use r2r::Context;
use r2r::QosProfile;
use r2r_regular_markers::RegularMarkerServer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub static NODE_ID: &'static str = "simple_marker";

#[derive(Debug, Clone, PartialEq)]
pub struct FrameData {
    pub parent_frame_id: String,
    pub child_frame_id: String,
    pub transform: Transform,
    pub active: Option<bool>,
}

fn make_initial_tf() -> HashMap<String, FrameData> {
    let mut test_setup = HashMap::<String, FrameData>::new();

    test_setup.insert(
        "base_link".to_string(),
        FrameData {
            parent_frame_id: "world".to_string(),
            child_frame_id: "base_link".to_string(),
            transform: r2r::geometry_msgs::msg::Transform {
                translation: r2r::geometry_msgs::msg::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
                rotation: r2r::geometry_msgs::msg::Quaternion {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
            },
            active: Some(false),
        },
    );
    test_setup
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::create()?;
    let node = r2r::Node::create(context, "asdf", "")?;
    let arc_node = Arc::new(Mutex::new(node));

    // We need to publish a frame where the marker can be placed
    let broadcasted_frames = Arc::new(Mutex::new(make_initial_tf()));
    let arc_node_clone = arc_node.clone();
    let static_pub_timer = arc_node_clone
        .lock()
        .unwrap()
        .create_wall_timer(std::time::Duration::from_millis(20))?;
    let static_frame_broadcaster = arc_node_clone
        .lock()
        .unwrap()
        .create_publisher::<TFMessage>(
            "tf",
            QosProfile::transient_local(QosProfile::default()),
        )?;
    let broadcasted_frames_clone = broadcasted_frames.clone();
    tokio::task::spawn(async move {
        match static_frame_broadcaster_callback(
            static_frame_broadcaster,
            static_pub_timer,
            &broadcasted_frames_clone,
        )
        .await
        {
            Ok(()) => (),
            Err(e) => r2r::log_error!(NODE_ID, "Active frame broadcaster failed with: '{}'.", e),
        };
    });

    let arc_node_clone = arc_node.clone();
    let server = RegularMarkerServer::new("simple_marker", arc_node_clone);

    let mut marker = Marker::default();
    marker.header.frame_id = "base_link".to_string();
    marker.type_ = Marker::CUBE as i32;
    marker.scale.x = 0.45;
    marker.scale.y = 0.45;
    marker.scale.z = 0.45;
    marker.color.r = 0.0;
    marker.color.b = 0.5;
    marker.color.g = 0.5;
    marker.color.a = 1.0;
    marker.pose = Pose {
        position: Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        orientation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
    };

    // Insert the marker into the server
    server.insert("my_marker", marker);

    // Apply changes to publish updates
    server.apply_changes();

    // Keep the node alive
    let arc_node_clone: Arc<Mutex<r2r::Node>> = arc_node.clone();
    let handle = std::thread::spawn(move || loop {
        arc_node_clone
            .lock()
            .unwrap()
            .spin_once(std::time::Duration::from_millis(100));
    });

    r2r::log_info!(NODE_ID, "Node started.");

    // tokio::time::sleep(std::time::Duration::from_millis(5000)).await;

    // server.delete("my_marker");
    // server.apply_changes();

    handle.join().unwrap();

    Ok(())
}

// Broadcast static frames
async fn static_frame_broadcaster_callback(
    publisher: r2r::Publisher<TFMessage>,
    mut timer: r2r::Timer,
    frames: &Arc<Mutex<HashMap<String, FrameData>>>,
    // node_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut x = 0.0;
    loop {
        x = x + 0.001;
        let mut clock = r2r::Clock::create(r2r::ClockType::RosTime).unwrap();
        let now = clock.get_now().unwrap();
        let time_stamp = r2r::Clock::to_builtin_time(&now);

        let transforms_local = frames.lock().unwrap().clone();
        let mut updated_transforms = vec![];

        transforms_local.iter().for_each(|(_, v)| match v.active {
            Some(false) => {
                updated_transforms.push(TransformStamped {
                    header: Header {
                        stamp: time_stamp.clone(),
                        frame_id: v.parent_frame_id.clone(),
                    },
                    child_frame_id: v.child_frame_id.clone(),
                    transform: Transform {
                        translation: Vector3 {
                            x,
                            y: v.transform.translation.y.clone(),
                            z: v.transform.translation.z.clone(),
                        },
                        rotation: v.transform.rotation.clone(),
                    },
                });
            }
            Some(true) | None => (),
        });

        let msg = TFMessage {
            transforms: updated_transforms,
        };

        match publisher.publish(&msg) {
            Ok(()) => (),
            Err(e) => {
                r2r::log_error!(
                    NODE_ID,
                    "Static broadcaster failed to send a message with: '{}'",
                    e
                );
            }
        };
        timer.tick().await?;
    }
}
