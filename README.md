# r2r_regular_markers

The `r2r_regular_markers` is a Rust module that manages and publishes markers using ROS2. It allows you to add, modify, and delete markers, which are periodically published as `MarkerArray` messages on a specified ROS topic.

## Features

- **Add Markers**: Insert new markers or update existing ones by name.
- **Delete Markers**: Remove markers by their unique names.
- **Apply Changes**: Commit pending updates to be published.
- **Periodic Publishing**: Automatically publishes markers at regular intervals.
- **Thread-Safe**: Utilizes `Arc` and `Mutex` for safe concurrent access.

## How It Works

- **Marker Management**: Maintains a list of active markers and pending updates.
- **Pending Updates**: Changes are first added to a pending updates list.
- **Apply Changes**: Calling `apply_changes` commits pending updates to the active marker list.
- **Periodic Publishing**: A background task publishes the active markers at regular intervals (every 20 milliseconds by default).

## Getting Started

### Prerequisites

- Rust programming language
- ROS2 environment
- [`r2r`](https://crates.io/crates/r2r) crate for ROS2 communication
- [`tokio`](https://crates.io/crates/tokio) for asynchronous operations

### Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
r2r = "0.9.0"
r2r_regular_markers = { git = "https://github.com/sequenceplanner/r2r_regular_markers", tag = "v0.0.1" }
tokio = { version = "1.36.0", features = ["full"] }

```

### Usage

#### Creating a RegularMarkerServer

```rust
let context = Context::create()?;
let node = r2r::Node::create(context, "my_marker", "")?;
let arc_node = Arc::new(Mutex::new(node));
let server = RegularMarkerServer::new("my_ns", "my_topic", arc_node);
```

#### Adding a Marker
```rust
let mut marker = Marker::default();
// Configure your marker properties here
marker_server.insert("unique_marker_name", marker);
```

#### Deleting a Marker
```rust
marker_server.delete("unique_marker_name");
```

#### Applying Changes
```rust
marker_server.apply_changes();
```



## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements.

## License

This project is open-source and available under the [TODO: License](LICENSE).