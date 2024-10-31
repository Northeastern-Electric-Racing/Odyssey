use std::{fs, time::Duration};

use sysinfo::{Components, MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace, warn};

use crate::PublishableMessage;

/// sender of the messages
pub async fn collect_data(
    cancel_token: CancellationToken,
    mqtt_sender_tx: Sender<PublishableMessage>,
) {
    // create requisites
    let sys = Mutex::new(System::new_all());
    let mut sys_l = sys.lock().await;
    sys_l.refresh_all();

    // for CPU temp
    let mut components = Components::new_with_refreshed_list();
    let mut temperature_component = components
        .iter_mut()
        .find(|x| x.label() == "coretemp Core 0");

    // for broker CPU
    let mut pid = match fs::read_to_string("/var/run/mosquitto.pid") {
        Ok(p) => p,
        Err(_) => {
            warn!("Could not read mosquitto PID, using 1");
            "1".to_string()
        }
    };
    if pid.ends_with('\n') {
        pid.pop();
    }
    let pid_clean = str::parse::<u32>(&pid).unwrap_or_else(|a| {
        warn!("Could not parse mosquitto pid, using 1, error: {}", a);
        1
    });
    drop(sys_l);

    // STEP 1: add a refresh rates for the message

    // on board
    let mut cpu_temp_int = tokio::time::interval(Duration::from_secs(2));
    let mut cpu_usage_int = tokio::time::interval(Duration::from_millis(300)); // minimum 200ms. both broker and global
    let mut mem_avail_int = tokio::time::interval(Duration::from_secs(1));

    loop {
        let msgs = tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Shutting down MQTT processor!");
                break;
            },
            // STEP 2: add a block to gather the data and send the message, include topic and unit blocks
            _ = cpu_temp_int.tick() => {
                const TOPIC: &'static str = "TPU/OnBoard/CpuTemp";
                const UNIT: &'static str = "celsius";

                let value = match temperature_component {
                    Some(ref mut v) => {
                        v.refresh();
                        v.temperature()
                    },
                    None => {
                        warn!("Could not find thermal sensor!");
                        continue;
                    }
                };

                vec![PublishableMessage{ topic: TOPIC, data: vec![value], unit: UNIT }]

            }
            _ = cpu_usage_int.tick() => {
                const TOPIC_C: &'static str = "TPU/OnBoard/CpuUsage";
                const UNIT_C: &'static str = "%";

                const TOPIC_B: &'static str = "TPU/OnBoard/BrokerCpuUsage";
                const UNIT_B: &'static str = "%";

                let mut sys_l = sys.lock().await;

                sys_l.refresh_cpu_usage();
                sys_l.refresh_processes(ProcessesToUpdate::Some(&[Pid::from_u32(pid_clean)]),
                        true);

                let process = sys_l.process(Pid::from_u32(pid_clean)).unwrap_or_else(|| {
                    warn!("Could not find mosquitto from PID, using 1");
                    sys_l.process(Pid::from(1)).unwrap()
                });
                trace!("Using process: {:?}", process.name());

                vec![
                    PublishableMessage{ topic: TOPIC_C, data: vec![sys_l.global_cpu_usage()], unit: UNIT_C },
                PublishableMessage{ topic: TOPIC_B, data: vec![process.cpu_usage()], unit: UNIT_B }]
            },
            _ = mem_avail_int.tick() => {
                const TOPIC: &'static str = "TPU/OnBoard/MemAvailable";
                const UNIT: &'static str = "MB";

                let mut sys_l = sys.lock().await;
                sys_l.refresh_memory_specifics(MemoryRefreshKind::new().with_ram());

                vec![PublishableMessage { topic: TOPIC, data: vec![sys_l.free_memory() as f32 / 1e6], unit: UNIT}]
            }
        };
        for msg in msgs {
            let Ok(_) = mqtt_sender_tx.send(msg).await else {
                warn!("Could not send mpsc msg to mqtt send");
                continue;
            };
        }
    }
}
