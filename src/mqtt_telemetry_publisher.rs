use rclrs::{CreateBasicExecutor, RclrsErrorFilter};
mod config;

use anyhow::Result;
use config::Config;
use ros_env::std_msgs::msg::String as StringMsg;
use rumqtt::{MqttClient, MqttOptions, QoS};
use std::{
    fs,
    sync::{Arc, Mutex},
};

struct MqttTelemetryPublisherNode {
    _node: rclrs::Node,
    _publisher: rclrs::Publisher<StringMsg>,
    _subscription: rclrs::Subscription<StringMsg>,
    _mqtt_client: Arc<Mutex<MqttClient>>,
}

impl MqttTelemetryPublisherNode {
    fn new(executor: &rclrs::Executor) -> Result<Self> {
        let node = executor.create_node("mqtt_publisher")?;
        let path_param = node
            .declare_parameter::<Arc<str>>("path_for_config")
            .mandatory()
            .expect("path_for_config must be set");

        let config = Config::try_from_path(&path_param.get()).expect("Unable to read config");

        let mqtt_options = {
            let ca_bytes = fs::read(config.rootCAPath)?;
            let cert_bytes = fs::read(config.certificatePath)?;
            let key_bytes = fs::read(config.privateKeyPath)?;
            let options = MqttOptions::new(config.clientID.clone(), config.endpoint, 8883)
                .set_ca(ca_bytes)
                .set_client_auth(cert_bytes, key_bytes);
            Ok::<MqttOptions, anyhow::Error>(options)
        }?;

        let (mqtt_client, _) = MqttClient::start(mqtt_options).unwrap();
        println!("Created MQTT client with ID {}.", &config.clientID);
        let _mqtt_client = Arc::new(Mutex::new(mqtt_client));
        let mqtt_cb = Arc::clone(&_mqtt_client);

        let _publisher = node.create_publisher::<StringMsg>("mock_telemetry")?;
        let _subscription = node.create_subscription("mock_telemetry", move |msg: StringMsg| {
            println!("Received data to republish: {}", msg.data);
            mqtt_cb
                .lock()
                .unwrap()
                .publish(
                    "ros2_mock_telemetry_topic",
                    QoS::AtLeastOnce,
                    false,
                    msg.data,
                )
                .expect("Could not publish payload!");
        })?;

        Ok(Self {
            _node: node,
            _publisher,
            _subscription,
            _mqtt_client,
        })
    }
}

fn main() -> Result<()> {
    let context = rclrs::Context::default_from_env()?;
    let mut executor = context.create_basic_executor();
    let _mqtt_telemetry = MqttTelemetryPublisherNode::new(&executor)?;
    executor.spin(rclrs::SpinOptions::default()).first_error()?;
    Ok(())
}
