// Copyright (C) 2022 - 2023 Wilfred Bos
// Licensed under the GNU GPL v3 license. See the LICENSE file for the terms and conditions.

mod device_state;
mod settings;
mod sid_device_listener;
mod sid_device_server;

use std::{thread, time::Duration};
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use async_broadcast::{broadcast, Receiver, Sender};
use parking_lot::Mutex;
use single_instance::SingleInstance;

use settings::Settings;
use sid_device_server::SidDeviceServer;

use crate::device_state::DeviceState;
use crate::settings::Config;
use crate::sid_device_listener::SidDeviceListener;

type SidDeviceChannel = (Sender<(SettingsCommand, Option<i32>)>, Receiver<(SettingsCommand, Option<i32>)>);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SettingsCommand {
    SetAudioDevice,
    EnableDigiboost,
    DisableDigiboost,
    FilterBias6581
}

fn main() {
    let instance = SingleInstance::new("sid-device").unwrap();
    if !instance.is_single() {
        println!("ERROR: SID Device is already running\r");
        exit(1);
    }

    let (mut device_sender, device_receiver):SidDeviceChannel = broadcast(1);
    device_sender.set_overflow(true);

    let settings = Arc::new(Mutex::new(Settings::new()));

    let device_state = start_sid_device_thread(device_receiver, &settings);
    start_sid_device_detect_thread(&device_state, &settings);

    loop {

    }
}

fn start_sid_device_thread(receiver: Receiver<(SettingsCommand, Option<i32>)>, settings: &Arc<Mutex<Settings>>) -> DeviceState {
    let device_state = DeviceState::new();

    let _sid_device_thread = thread::spawn({
        let settings_clone = settings.clone();
        let device_state = device_state.clone();

        move || {
            sid_device_loop(receiver, &settings_clone, device_state);
        }
    });

    device_state
}

fn sid_device_loop(receiver: Receiver<(SettingsCommand, Option<i32>)>, settings: &Arc<Mutex<Settings>>, device_state: DeviceState) {
    while device_state.restart.load(Ordering::SeqCst) {
        while device_state.error.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(500));
        }

        let mut sid_device_server = SidDeviceServer::new(settings.lock().get_config());

        device_state.init();

        let allow_external_connections = settings.lock().get_config().lock().allow_external_connections;

        let server_result = sid_device_server.start(allow_external_connections,receiver.clone(), device_state.device_ready.clone(), device_state.quit.clone());

        if let Err(server_result) = server_result {
            println!("ERROR: {server_result}\r");
            device_state.set_error(server_result);
        }
    }

    device_state.stopped.store(true, Ordering::SeqCst);
}

fn start_sid_device_detect_thread(device_state: &DeviceState, settings: &Arc<Mutex<Settings>>) {
    let _sid_device_detect_thread = thread::spawn({
        let device_state_clone = device_state.clone();
        let settings_clone = settings.clone();
        move || {
            match SidDeviceListener::new() {
                Ok(listener) => sid_device_detect_loop(listener, &settings_clone, &device_state_clone),
                Err(err) => println!("ERROR: {err}\r")
            }
        }
    });
}

fn sid_device_detect_loop(listener: SidDeviceListener, settings: &Arc<Mutex<Settings>>, device_state: &DeviceState) {
    loop {
        if device_state.stopped.load(Ordering::SeqCst) {
            break;
        }

        match listener.detect_client() {
            Ok(client) => {
                if let Some(client) = client {
                    let allow_external_connections = settings.lock().get_config().lock().allow_external_connections;

                    if allow_external_connections {
                        println!("Client detected with address: {}:{}", client.ip_address, client.port);

                        if let Err(err) = listener.respond(&client) {
                            println!("ERROR: Response could not be send: {err}\r");
                        }
                    }
                }
            },
            Err(err) => println!("ERROR: {err}\r")
        }
    }
}