use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use rumqttc::v5::AsyncClient;
use tokio::{
    signal,
    sync::{mpsc, RwLock},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tpu_telemetry::{
    mqtt_handler::{MqttProcessor, MqttProcessorOptions},
    numerical::collect_data,
    visual::{run_save_pipeline, OverlayOpts, SavePipelineOpts},
    PublishableMessage,
};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

/// ody-visual command line arguments
#[derive(Parser, Debug)]
#[command(version)]
struct VisualArgs {
    /// Enable data mode
    #[arg(short = 'd', long, env = "TPU_TELEMETRY_DATA_ENABLE")]
    data: bool,

    /// Enable video mode
    #[arg(short = 'v', long, env = "TPU_TELEMETRY_VIDEO_ENABLE")]
    video: bool,

    /// The video file
    #[arg(short = 'l', long, env = "TPU_TELEMETRY_VIDEO_FILE")]
    video_uri: String,

    /// The MQTT/Siren URL
    #[arg(
        short = 'u',
        long,
        default_value = "localhost:1883",
        env = "TPU_TELEMETRY_SIREN_URL"
    )]
    mqtt_url: String,

    /// The MQTT topic to get the data from if overlay is wanted
    #[arg(short = 't', long, env = "TPU_TELEMETRY_SIREN_TOPIC")]
    mqtt_topic: Option<String>,

    /// The output folder of videos, no trailing slash
    #[arg(short = 'f', long, env = "TPU_TELEMETRY_OUTPUT_FOLDER")]
    output_folder: String,
}

#[tokio::main]
async fn main() {
    let cli = VisualArgs::parse();

    println!("Initializing tpu telemetry...");
    println!("Initializing fmt subscriber");
    // construct a subscriber that prints formatted traces to stdout
    // if RUST_LOG is not set, defaults to loglevel INFO
    let subscriber = tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_ansi(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).expect("Could not init tracing");

    // channel to pass the mqtt data
    // TODO tune buffer size
    let (mqtt_sender_tx, mqtt_sender_rx) = mpsc::channel::<PublishableMessage>(1000);

    let task_tracker = TaskTracker::new();
    let token = CancellationToken::new();

    // time is wrong for a while upon boot.  hold on until it is OK
    while !SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .is_ok_and(|time| time > Duration::from_millis(1730247194876))
    {
        info!("Waiting for good time");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // use the passed in folder
    let save_location = format!(
        "{}/frontcam-{}-ner24.avi",
        cli.output_folder,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let gst_token = token.clone();
    let (overlay_opts, mqtt_recv) = match cli.mqtt_topic {
        Some(topic) => {
            let data_rw = Arc::new(RwLock::new(0f32));
            (
                Some(OverlayOpts {
                    overlay_label: topic.clone(),
                    data_source: Arc::clone(&data_rw),
                }),
                Some((topic, data_rw)),
            )
        }
        None => (None, None),
    };
    if cli.video {
        task_tracker.spawn(run_save_pipeline(
            gst_token,
            SavePipelineOpts {
                video: cli.video_uri.clone(),
                save_location,
            },
            overlay_opts,
        ));
    }

    info!("Running MQTT processor");
    let (recv, opts) = MqttProcessor::new(
        token.clone(),
        mqtt_sender_rx,
        MqttProcessorOptions {
            mqtt_path: cli.mqtt_url,
            mqtt_recv,
        },
    );
    let (client, eventloop) = AsyncClient::new(opts, 600);
    let client_sharable: Arc<AsyncClient> = Arc::new(client);
    task_tracker.spawn(recv.process_mqtt(client_sharable.clone(), eventloop));

    if cli.data {
        info!("Running TPU data collector");
        task_tracker.spawn(collect_data(token.clone(), mqtt_sender_tx.clone()));
    }

    task_tracker.close();

    info!("Initialization complete, ready...");
    info!("Use Ctrl+C or SIGINT to exit cleanly!");

    // listen for ctrl_c, then cancel, close, and await for all tasks in the tracker.  Other tasks cancel vai the default tokio system
    signal::ctrl_c()
        .await
        .expect("Could not read cancellation trigger (ctr+c)");
    info!("Received exit signal, shutting down!");
    token.cancel();
    task_tracker.wait().await;
}
