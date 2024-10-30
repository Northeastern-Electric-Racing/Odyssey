use std::sync::Arc;

// DO NOT REMOVE
use gstreamer::prelude::*;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, trace, warn};

pub struct SavePipelineOpts {
    pub video: String,
    pub save_location: String,
}

pub struct OverlayOpts {
    pub overlay_label: String,
    pub data_source: Arc<RwLock<f32>>,
}

/// Run a save pipeline on the items
pub async fn run_save_pipeline(
    cancel_token: CancellationToken,
    vid_opts: SavePipelineOpts,
    overlay_opts: Option<OverlayOpts>,
) -> Result<(), gstreamer::glib::BoolError> {
    debug!("Initializing gstreamer...");
    gstreamer::init().expect("Could not init gstreamer");
    // let pipeline = gstreamer::parse::launch(&format!(
    //     "v4l2src device={} ! decodebin ! videoconvert ! timeoverlay show-times-as-dates=true datetime-epoch=g_date_time_new_now_local ! autovideosink",
    //     cli.video
    // ))

    let activate_overlay = overlay_opts.is_some();

    debug!("Setting up pipeline!");
    let source = gstreamer::ElementFactory::make("v4l2src")
        .name("source")
        .property("device", vid_opts.video)
        .build()?;
    let b = gstreamer::ElementFactory::make("videoconvert")
        .name("video")
        .build()?;
    let c = gstreamer::ElementFactory::make("timeoverlay")
        .property("show-times-as-dates", true)
        .property(
            "datetime-epoch",
            gstreamer::DateTime::new_now_local_time()
                .expect("Could not construct local time")
                .to_g_date_time()?,
        )
        .build()?;
    let d = gstreamer::ElementFactory::make("textoverlay")
        .property("text", "Northeastern Electric Racing")
        .property_from_str("valignment", "bottom") // bottom
        .property_from_str("halignment", "right") // right
        .build()?;
    let text = gstreamer::ElementFactory::make("textoverlay")
        .property("text", "")
        .property_from_str("valignment", "bottom") // bottom
        .property_from_str("halignment", "left") // right
        .build()?;

    let e = gstreamer::ElementFactory::make("avimux").build()?;
    let sink = gstreamer::ElementFactory::make("filesink")
        .property("location", vid_opts.save_location)
        .build()?;

    let pipeline = gstreamer::Pipeline::with_name("ody-visual");
    pipeline.add_many([&source, &sink, &b, &c, &d, &e, &text])?;

    gstreamer::Element::link_many([&source, &b, &c, &d, &text, &e, &sink])?;

    println!("Playing");
    pipeline
        .set_state(gstreamer::State::Playing)
        .expect("Could not begin");
    let bus = pipeline.bus().expect("Could not create bus");

    // exit loop when system terms
    while !cancel_token.is_cancelled() {
        {
            use gstreamer::MessageView;

            // do not update overlay if EOS or error is recieved
            let Some(msg) = bus.pop() else {
                // TODO fix possible issue where overlay empty but activate_pipeline true, in fact yeet activate pipeline
                if activate_overlay {
                    trace!("Updating overlay property!");
                    text.set_property(
                        "text",
                        format!(
                            "{}: {}",
                            overlay_opts.as_ref().unwrap().overlay_label,
                            overlay_opts.as_ref().unwrap().data_source.read().await
                        ),
                    );
                }
                // go to next loop when label updated
                continue;
            };

            match msg.view() {
                MessageView::Eos(..) => {
                    warn!("Done with streaming prematurely!");
                    break;
                }
                MessageView::Error(err) => {
                    warn!(
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
    // end playing and cleanup, vital for cleaning the sink file
    pipeline.send_event(gstreamer::event::Eos::new());
    // iterate through any messages related to cleanup
    for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
        use gstreamer::MessageView;
        match msg.view() {
            MessageView::Eos(..) => {
                info!("Done with streaming!");
                break;
            }
            MessageView::Error(err) => {
                warn!(
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

    info!("Shutting down pipeline!");
    pipeline
        .set_state(gstreamer::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");

    Ok(())
}
