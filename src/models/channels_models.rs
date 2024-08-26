use crate::models::data_thread_models::{DataThreadCommand, DataThreadResponse};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Channels {
    pub(crate) route_to_data_sender: Sender<DataThreadCommand>,
    pub(crate) data_to_route_receiver: Arc<Mutex<Receiver<DataThreadResponse>>>,
}
