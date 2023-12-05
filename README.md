# AWS IoT Node

This package is an example of a ROS2 node written in Rust.

It will use X.509 certificates to connect to AWS IoT Core, then forward any ROS2 messages received on `/ros2_mock_telemetry_topic` up to IoT Core.

## Setup

### Tool Installation

If you do not have Rust installed, you can easily install it with [Rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

To be able to build ROS2 Rust, there are some further prerequisites. These steps are taken from the [ROS2 Rust repository](https://github.com/ros2-rust/ros2_rust#sounds-great-how-can-i-try-this-out). Execute the following:

```bash
sudo apt install -y git libclang-dev python3-pip python3-vcstool # libclang-dev is required by bindgen
# Install these plugins for cargo and colcon:
cargo install --debug cargo-ament-build  # --debug is faster to install
pip install git+https://github.com/colcon/colcon-cargo.git
pip install git+https://github.com/colcon/colcon-ros-cargo.git
```

### Source Code

You can clone this repository directly into your ROS2 workspace. The workspace can be set up following steps 1 and 2 of [this guide](https://docs.ros.org/en/foxy/Tutorials/Beginner-Client-Libraries/Creating-A-Workspace/Creating-A-Workspace.html).

Inside the `src` directory, clone this repository, the ROS2 Rust repository, and the AWS IoT Connectivity Samples for ROS2:

```bash
cd ~/ros2_ws/src
git clone https://github.com/mikelikesrobots/aws-iot-node-rust.git
git clone https://github.com/ros2-rust/ros2_rust.git
git clone https://github.com/aws-samples/aws-iot-robot-connectivity-samples-ros2.git
```

ROS2 Rust has further repositories to check out, which can be done automatically using:

```bash
cd ~/ros2_ws
vcs import src < src/ros2_rust/ros2_rust_humble.repos
```

You should now follow the instructions in `aws-iot-robot-connectivity-samples-ros2` to generate a valid X.509 certificate with the correct permissions. This sample uses the same permissions as the connectivity sample.

### Building the workspace

The workspace can now be built. It takes around 10m to build ROS2 Rust, which should only need to be done once. Following that, changes to the code from this repository can be built very quickly. To build the workspace, execute:

```bash
cd ~/ros2_ws
colcon build
source install/setup.bash

# After first build:
colcon build --packages-select aws_iot_node
```

Now, any changes that are made to this repository can be built and tested with `cargo` commands, such as:

```bash
cargo build
cargo run --bin mock-telemetry
```

## Running

To run the example, you will need the `IOT_CONFIG_FILE` variable set from the `aws-iot-robot-connectivity-samples-ros2` repository.

Open two terminals. In each terminal, source the workspace, then run one of the two nodes as follows:

```bash
source ~/ros2_ws/install/setup.bash  # Both terminals
ros2 run aws_iot_node mqtt-telemetry --ros-args --param path_for_config:=$IOT_CONFIG_FILE  # One terminal
ros2 run aws_iot_node mock-telemetry  # Other terminal
```

You should now see messages appearing in the MQTT test client in AWS IoT Core.
