use rclrs::RclrsError;
use rand::Rng;
use serde::Serialize;
use std::sync::Arc;
use std_msgs::msg::String as StringMsg;

#[derive(Serialize, Debug)]
struct MockTelemetryPoint {
    battery: f32,
    velocity: f32,
}
struct MockTelemetryPublisherNode {
    node: Arc<rclrs::Node>,
    _publisher: Arc<rclrs::Publisher<StringMsg>>,
}

fn round2dp(x: f32) -> f32 {
    (x * 100.0).round() / 100.0
}

impl MockTelemetryPublisherNode {
    fn new(context: &rclrs::Context) -> Result<Self, rclrs::RclrsError> {
        let node = rclrs::Node::new(context, "mock_telemetry_publisher")?;
        let _publisher = node.create_publisher("mock_telemetry", rclrs::QOS_PROFILE_DEFAULT)?;

        Ok(Self { node, _publisher })
    }

    fn publish_mock_data(&self) -> Result<(), rclrs::RclrsError> {
        let mut rng = rand::thread_rng();
        let data = MockTelemetryPoint {
            battery: round2dp(rng.gen_range(85f32 .. 90f32)),
            velocity: round2dp(rng.gen_range(3.0f32 .. 4.0f32)),
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
    let context = rclrs::Context::new(std::env::args())?;
    let mock_telemetry = Arc::new(MockTelemetryPublisherNode::new(&context)?);
    let mock_telemetry_other_thread = Arc::clone(&mock_telemetry);
    std::thread::spawn(move || -> Result<(), rclrs::RclrsError> {
        loop {
            use std::time::Duration;
            std::thread::sleep(Duration::from_millis(1000));
            mock_telemetry_other_thread.publish_mock_data()?;
        }
    });
    rclrs::spin(mock_telemetry.node.clone())
}
