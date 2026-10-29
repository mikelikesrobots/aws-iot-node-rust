use rclrs::RclrsError;
use rclrs::{CreateBasicExecutor, RclrsErrorFilter};
use ros_env::std_msgs::msg::String as StringMsg;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, Debug)]
struct MockTelemetryPoint {
    battery: f32,
    velocity: f32,
}
struct MockTelemetryPublisherNode {
    _node: rclrs::Node,
    _publisher: rclrs::Publisher<StringMsg>,
}

fn round2dp(x: f32) -> f32 {
    (x * 100.0).round() / 100.0
}

impl MockTelemetryPublisherNode {
    fn new(executor: &rclrs::Executor) -> Result<Self, rclrs::RclrsError> {
        let node = executor.create_node("mock_telemetry_publisher")?;
        let _publisher = node.create_publisher("mock_telemetry")?;

        Ok(Self {
            _node: node,
            _publisher,
        })
    }

    fn publish_mock_data(&self) -> Result<(), rclrs::RclrsError> {
        let data = MockTelemetryPoint {
            battery: round2dp(rand::random_range(85f32..90f32)),
            velocity: round2dp(rand::random_range(3.0f32..4.0f32)),
        };
        let serialized = serde_json::to_string(&data).map_err(|_| RclrsError::RclError {
            code: rclrs::RclReturnCode::Error,
            msg: None,
        })?;
        let msg = StringMsg { data: serialized };
        self._publisher.publish(&msg)?;
        Ok(())
    }
}

fn main() -> Result<(), rclrs::RclrsError> {
    let context = rclrs::Context::default_from_env()?;
    let mut executor = context.create_basic_executor();
    let mock_telemetry = Arc::new(MockTelemetryPublisherNode::new(&executor)?);
    let mock_telemetry_other_thread = Arc::clone(&mock_telemetry);
    std::thread::spawn(move || -> Result<(), rclrs::RclrsError> {
        loop {
            use std::time::Duration;
            std::thread::sleep(Duration::from_millis(1000));
            mock_telemetry_other_thread.publish_mock_data()?;
        }
    });
    executor.spin(rclrs::SpinOptions::default()).first_error()
}
