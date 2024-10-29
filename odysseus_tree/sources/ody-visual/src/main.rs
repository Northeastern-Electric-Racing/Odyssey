use std::sync::{atomic::AtomicBool, Arc};

use clap::Parser;
use gstreamer::prelude::*;
use serde::Deserialize;

/// ody-visual command line arguments
#[derive(Parser, Debug)]
#[command(version)]
struct VisualArgs {
    /// The video file
    #[arg(short = 'v', long, env = "ODY_VISUAL_VIDEO_FILE")]
    video: String,

    /// The scylla URL
    #[arg(
        short = 'u',
        long,
        default_value = "localhost:8000",
        env = "ODY_VISUAL_SCYLLA_URL"
    )]
    scylla_url: String,
}
#[derive(Deserialize, Debug, PartialEq)]
pub struct PublicRun {
    pub id: i32,
    #[serde(rename = "locationName")]
    pub location_name: String,
    #[serde(rename = "driverName")]
    pub driver_name: String,
    #[serde(rename = "systemName")]
    pub system_name: String,
    #[serde(rename = "time")]
    pub time_ms: i64,
}

fn run_save_pipeline(
    end_signal: Arc<AtomicBool>,
    video: String,
    save_location: String,
) -> Result<String, gstreamer::glib::BoolError> {
    println!("Initializing gstreamer...");
    gstreamer::init().expect("Could not init gstreamer");
    // let pipeline = gstreamer::parse::launch(&format!(
    //     "v4l2src device={} ! decodebin ! videoconvert ! timeoverlay show-times-as-dates=true datetime-epoch=g_date_time_new_now_local ! autovideosink",
    //     cli.video
    // ))

    println!("Setting up pipeline!");
    let source = gstreamer::ElementFactory::make("v4l2src")
        .name("source")
        .property("device", video)
        .build()?;
    // let a = gstreamer::ElementFactory::make("decodebin").name("decode")
    //     .build()
    //     .unwrap();
    let b = gstreamer::ElementFactory::make("videoconvert")
        .name("video")
        .build()?;
    let c = gstreamer::ElementFactory::make("timeoverlay")
        .property("show-times-as-dates", true)
        .property(
            "datetime-epoch",
            gstreamer::DateTime::new_now_local_time()
                .unwrap()
                .to_g_date_time()?,
        )
        .build()?;
    let d = gstreamer::ElementFactory::make("textoverlay")
        .property("text", "Northeastern Electric Racing")
        .property_from_str("valignment", "bottom") // bottom
        .property_from_str("halignment", "right") // right
        .build()?;
    // let sink = gstreamer::ElementFactory::make("autovideosink")
    //     .name("sink")
    //     .build()
    //     .unwrap();
    let e = gstreamer::ElementFactory::make("avimux").build()?;
    let sink = gstreamer::ElementFactory::make("filesink")
        .property("location", save_location)
        .build()?;

    let pipeline = gstreamer::Pipeline::with_name("ody-visual");
    pipeline.add_many([&source, &sink, &b, &c, &d, &e])?;

    gstreamer::Element::link_many([&source, &b, &c, &d, &e, &sink])?;

    println!("Playing");
    pipeline
        .set_state(gstreamer::State::Playing)
        .expect("Could not begin");
    let bus = pipeline.bus().expect("Could not create bus");
    while !end_signal.load(std::sync::atomic::Ordering::Relaxed) {
        {
            use gstreamer::MessageView;

            let Some(msg) = bus.pop() else {
                continue;
            };

            match msg.view() {
                MessageView::Eos(..) => {
                    println!("Done with streaming prematurely!");
                    break;
                }
                MessageView::Error(err) => {
                    println!(
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    break;
                }
                _ => (),
            }
        }
    }
    pipeline.send_event(gstreamer::event::Eos::new());
    for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
        use gstreamer::MessageView;
        match msg.view() {
            MessageView::Eos(..) => {
                println!("Done with streaming!");
                break;
            }
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            _ => (),
        }
    }
    // Shutdown pipeline
    println!("Shutting down pipeline!");
    pipeline
        .set_state(gstreamer::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");

    Ok("Done".to_owned())
}

fn main() {
    let cli = VisualArgs::parse();

    let stop = Arc::new(AtomicBool::new(false));
    for signal in signal_hook::consts::TERM_SIGNALS {
        signal_hook::flag::register(*signal, Arc::clone(&stop)).unwrap();
    }

    let res = reqwest::blocking::get(format!("http://{}/runs", cli.scylla_url)).unwrap();
    let run: Vec<PublicRun> = serde_json::from_str(&res.text().unwrap()).unwrap();

    let save_location = format!("run{}-ner.avi", run.last().unwrap().id);
    let gst_mgr =
        std::thread::spawn(move || run_save_pipeline(stop, cli.video, save_location.to_owned()));

    let gst_res = gst_mgr.join().expect("Join deadlock?");
    if gst_res.is_err() {
        println!("Error in gst thread: {:?}", gst_res);
    }
}
