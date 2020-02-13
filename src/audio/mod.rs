extern crate cpal;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use std::thread;
use std::sync::{Mutex, mpsc::{SyncSender, sync_channel}};

#[derive(Debug)]
pub struct AudioHandler {
    sender: Option<SyncSender<i16>>,
    samples: u128,
}

impl AudioHandler {
    pub fn new() -> AudioHandler {
        return AudioHandler {
            sender: None,
            samples: 0,
        };
    }

    fn aquire(&mut self, rt: SyncSender<i16>) {
        self.sender = Some(rt);
    }

    pub fn handle(&mut self, amplitude: i16) {
        self.samples += 1;
        match &self.sender {
            Some(rt) => { rt.send(amplitude).unwrap() },
            None => {},
        }
    }

    pub fn spawn_audio(&mut self) {
        let target_freq = 48_000;
        let (tx_data, rx_data) = sync_channel::<i16>(6);
        let (tx_confirm, rx_confirm) = sync_channel::<bool>(1);

        thread::spawn(move || {
            let host = cpal::default_host();
            let event_loop = host.event_loop();
            let (stream_id, num_channels) = match get_audio_config(target_freq, &host, &event_loop) {
                Ok(result) => {
                    println!("Spawning audio at freq {}", target_freq);
                    tx_confirm.send(true).unwrap();
                    result
                },
                Err(e) => {
                    println!("Failed to spawn audio: {}", e);
                    tx_confirm.send(false).unwrap();
                    return;
                }
            };

            let rx_data = Mutex::new(rx_data);
            event_loop.play_stream(stream_id.clone()).unwrap();

            event_loop.run(move |id, result| {
                let data = match result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", id, err);
                        return;
                    }
                };

                let rx_data = rx_data.lock().unwrap();
                match data {
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                        for sample in buffer.chunks_mut(num_channels) {
                            let value = rx_data.recv().unwrap();
                            for out in sample.iter_mut() {
                                *out = value;
                            }
                        }
                    },
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                        for sample in buffer.chunks_mut(num_channels) {
                            let value = map_to_unsigned(rx_data.recv().unwrap());
                            for out in sample.iter_mut() {
                                *out = value;
                            }
                        }
                    },
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                        for sample in buffer.chunks_mut(num_channels) {
                            let value = map_to_float(rx_data.recv().unwrap());
                            for out in sample.iter_mut() {
                                *out = value;
                            }
                        }
                    },
                    kind => {
                        panic!("unrecognised stream data kind");
                    }
                }
            });
        });

        if rx_confirm.recv().unwrap() {
            println!("audio speakers connected");
            self.sender = Some(tx_data);
        } else {
            println!("could not connect speakers");
        }
    }
}

fn get_audio_config(freq: u32, host: &cpal::Host, event_loop: &cpal::EventLoop) -> Result<(cpal::StreamId, usize), String> {
    let device = host.default_output_device().expect("failed to find a default output device");

    let formats = device.supported_output_formats().unwrap();
    let required_freq = cpal::SampleRate(freq);
    for supported in formats {
        println!("F= channels: {:?}, min: {:?}, max: {:?}, data: {:?}", supported.channels, supported.min_sample_rate, supported.max_sample_rate, supported.data_type);
        if supported.min_sample_rate < required_freq || supported.max_sample_rate > required_freq {
            continue;
        }

        let mut format = supported.with_max_sample_rate();
        format.sample_rate = required_freq;

        let stream_id = match event_loop.build_output_stream(&device, &format) {
            Ok(id) => id,
            Err(e) => {
                return Err(format!("Failed to build output stream: {}", e));
            }
        };
        let channels = usize::from(format.channels);

        return Ok((stream_id, channels));
    }

    return Err("Could not find compatible audio output".to_string());
}

fn map_to_float(val: i16) -> f32 {
    let float = f32::from(val);
    let max = f32::from(i16::max_value());
    return float / max;
}

fn map_to_unsigned(val: i16) -> u16 {
    return (u16::max_value() / 2).wrapping_add(val as u16);
}
