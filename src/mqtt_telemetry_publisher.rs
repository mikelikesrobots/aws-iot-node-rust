use rclrs::{CreateBasicExecutor, RclrsErrorFilter};
mod config;

use anyhow::{Context, Result};
use config::Config;
use ros_env::std_msgs::msg::String as StringMsg;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS, Transport};
use std::{fs, sync::Arc, thread, time::Duration};

struct MqttTelemetryPublisherNode {
    _node: rclrs::Node,
    _publisher: rclrs::Publisher<StringMsg>,
    _subscription: rclrs::Subscription<StringMsg>,
    _mqtt_client: Client,
}

impl MqttTelemetryPublisherNode {
    fn new(executor: &rclrs::Executor) -> Result<Self> {
        let node = executor.create_node("mqtt_publisher")?;
        let path_param = node
            .declare_parameter::<Arc<str>>("path_for_config")
            .mandatory()
            .expect("path_for_config must be set");

        let config = Config::try_from_path(&path_param.get()).expect("Unable to read config");

        let port = u16::try_from(config.port).context("MQTT port must be between 1 and 65535")?;
        anyhow::ensure!(port != 0, "MQTT port must be between 1 and 65535");
        let ca_bytes = fs::read(&config.rootCAPath).context("Unable to read MQTT root CA")?;
        let cert_bytes =
            fs::read(&config.certificatePath).context("Unable to read MQTT certificate")?;
        let key_bytes =
            fs::read(&config.privateKeyPath).context("Unable to read MQTT private key")?;
        let mut mqtt_options = MqttOptions::new(&config.clientID, &config.endpoint, port);
        mqtt_options.set_transport(Transport::tls(
            ca_bytes,
            Some((cert_bytes, key_bytes)),
            None,
        ));
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        let (mqtt_client, mut connection) = Client::new(mqtt_options, 100);
        let retry_delay = Duration::from_secs(config.retryWaitTime.max(1) as u64);
        // Polling drives TLS, publishes, acknowledgements, keep-alives and reconnects.
        thread::Builder::new()
            .name("mqtt-connection".into())
            .spawn(move || {
                for event in connection.iter() {
                    match event {
                        Ok(Event::Incoming(Packet::ConnAck(_))) => {
                            println!("Connected to MQTT broker")
                        }
                        Ok(_) => {}
                        Err(error) => {
                            eprintln!(
                                "MQTT connection error: {error}; retrying in {retry_delay:?}"
                            );
                            thread::sleep(retry_delay);
                        }
                    }
                }
            })?;
        println!("Created MQTT client with ID {}.", config.clientID);
        let mqtt_cb = mqtt_client.clone();

        let _publisher = node.create_publisher::<StringMsg>("mock_telemetry")?;
        let _subscription = node.create_subscription("mock_telemetry", move |msg: StringMsg| {
            println!("Received data to republish: {}", msg.data);
            // Never block the ROS executor or panic during a network outage.
            if let Err(error) = mqtt_cb.try_publish(
                "ros2_mock_telemetry_topic",
                QoS::AtLeastOnce,
                false,
                msg.data,
            ) {
                eprintln!("Dropping telemetry: MQTT queue is full or closed: {error}");
            }
        })?;

        Ok(Self {
            _node: node,
            _publisher,
            _subscription,
            _mqtt_client: mqtt_client,
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
