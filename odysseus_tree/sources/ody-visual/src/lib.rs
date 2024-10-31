pub mod mqtt_handler;
pub mod serverdata;
pub mod visual;
pub mod numerical;

#[derive(std::fmt::Debug)]
pub struct PublishableMessage {
    pub topic: &'static str,
    pub data: Vec<f32>,
    pub unit: &'static str,
}
