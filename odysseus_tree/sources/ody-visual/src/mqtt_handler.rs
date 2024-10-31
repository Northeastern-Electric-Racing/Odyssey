use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use protobuf::Message;
use rumqttc::v5::{
    mqttbytes::{v5::Packet, QoS},
    AsyncClient, Event, EventLoop, MqttOptions,
};
use tokio::sync::{mpsc::Receiver, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace, warn};

use crate::{serverdata, PublishableMessage};

/// The chief processor of incoming mqtt data, this handles
/// - mqtt state
/// - reception via mqtt and subsequent parsing
///
pub struct MqttProcessor {
    cancel_token: CancellationToken,
    mqtt_sender_rx: Receiver<PublishableMessage>,
    mqtt_recv: Option<(String, Arc<RwLock<f32>>)>,
}

/// processor options, these are static immutable settings
pub struct MqttProcessorOptions {
    /// URI of the mqtt server
    pub mqtt_path: String,
    /// MQTT topic and place to put data, or none
    pub mqtt_recv: Option<(String, Arc<RwLock<f32>>)>,
}

impl MqttProcessor {
    /// Creates a new mqtt receiver and sender
    pub fn new(
        cancel_token: CancellationToken,
        mqtt_sender_rx: Receiver<PublishableMessage>,
        opts: MqttProcessorOptions,
    ) -> (MqttProcessor, MqttOptions) {
        // create the mqtt client and configure it
        let mut mqtt_opts = MqttOptions::new(
            format!(
                "Ody-{:?}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis()
            ),
            opts.mqtt_path.split_once(':').expect("Invalid Siren URL").0,
            opts.mqtt_path
                .split_once(':')
                .unwrap()
                .1
                .parse::<u16>()
                .expect("Invalid Siren port"),
        );
        mqtt_opts
            .set_keep_alive(Duration::from_secs(20))
            .set_clean_start(false)
            .set_connection_timeout(3)
            //       .set_session_expiry_interval(Some(u32::MAX))
            .set_topic_alias_max(Some(600));

        (
            MqttProcessor {
                cancel_token,
                mqtt_sender_rx,
                mqtt_recv: opts.mqtt_recv,
            },
            mqtt_opts,
        )
    }

    /// This handles the reception of mqtt messages, will not return
    /// * `eventloop` - The eventloop returned by ::new to connect to.  The loop isnt sync so this is the best that can be done
    /// * `client` - The async mqttt v5 client to use for subscriptions
    pub async fn process_mqtt(mut self, client: Arc<AsyncClient>, mut eventloop: EventLoop) {
        debug!("Subscribing to siren with inputted topic");
        if self.mqtt_recv.as_ref().is_some() {
            client
                .subscribe(
                    self.mqtt_recv.as_ref().unwrap().0.clone(),
                    rumqttc::v5::mqttbytes::QoS::ExactlyOnce,
                )
                .await
                .expect("Could not subscribe to Siren");
        }

        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    debug!("Shutting down MQTT processor!");
                    break;
                },
                msg = eventloop.poll() => match msg {
                    Ok(Event::Incoming(Packet::Publish(msg))) => {
                        let Ok(res) = serverdata::ServerData::parse_from_bytes(&msg.payload) else {
                            warn!("Recieved unparsable mqtt message.");
                            continue;
                        };
                        match self.mqtt_recv {
                            Some(ref rcv) => {
                                let mut d = rcv.1.write().await;
                                *d = *res.values.first().unwrap_or(&0f32);
                            },
                            None => continue,
                        }
                    }
                    Err(e) => trace!("Recieved error: {}", e),
                    _ => {}
                },
                sendable = self.mqtt_sender_rx.recv() => {
                    match sendable {
                        Some(sendable) => {
                            trace!("Sending {:?}", sendable);
                            let mut payload = serverdata::ServerData::new();
                            payload.unit = sendable.unit.to_string();
                            payload.values = sendable.data;
                            payload.time_us =  SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .expect("Time went backwards").as_micros() as u64;
                            let Ok(bytes) = protobuf::Message::write_to_bytes(&payload) else {
                                warn!("Failed to serialize protobuf message!");
                                continue;
                            };
                            let Ok(_) = client.publish(sendable.topic, QoS::ExactlyOnce, false, bytes).await else {
                                warn!("Failed to send MQTT message!");
                                continue;
                            };
                        },
                        None => continue,
                    }
                }
            }
        }
    }
}
