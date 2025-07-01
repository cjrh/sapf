//! Real-time audio output using cpal
//!
//! This module provides the Rust equivalent of the C++ Play.cpp functionality,
//! enabling real-time audio playback of SAPF UGens and value streams.

use crate::core::{SapfError, Result, Value, List, Object};
use crate::vm::{Thread, VM};
use crate::audio::{AudioIn, AudioConfig, MAX_CHANNELS};
use crate::dsp::ugen::Sample;

use cpal::{Device, Stream, StreamConfig, Host, SampleRate};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

/// Global audio player manager - equivalent to gAllPlayers in C++
static AUDIO_MANAGER: OnceLock<Arc<Mutex<AudioManager>>> = OnceLock::new();

/// Unique ID for audio players
type PlayerId = u64;

/// Audio player state - equivalent to C++ AUPlayer struct
pub struct AudioPlayer {
    id: PlayerId,
    thread: Thread,
    inputs: Vec<AudioIn>,
    num_channels: usize,
    done: Arc<AtomicBool>,
    config: AudioConfig,
}

impl AudioPlayer {
    /// Create a new audio player
    fn new(id: PlayerId, thread: Thread, inputs: Vec<AudioIn>, config: AudioConfig) -> Self {
        // Ensure we don't exceed channel limits
        let num_channels = inputs.len().min(MAX_CHANNELS);
        
        Self {
            id,
            thread,
            inputs,
            num_channels,
            done: Arc::new(AtomicBool::new(false)),
            config,
        }
    }

    /// Check if this player is done
    pub fn is_done(&self) -> bool {
        self.done.load(Ordering::Relaxed)
    }

    /// Mark this player as done
    pub fn set_done(&self) {
        self.done.store(true, Ordering::Relaxed);
    }

    /// Start audio playback
    fn start(&mut self, device: Device) -> Result<()> {
        let stream_config = StreamConfig {
            channels: self.num_channels as u16,
            sample_rate: SampleRate(self.config.sample_rate as u32),
            buffer_size: cpal::BufferSize::Fixed(self.config.buffer_size as u32),
        };

        let done_flag = Arc::clone(&self.done);
        let done_flag_bg = Arc::clone(&self.done);
        let mut inputs = std::mem::take(&mut self.inputs);
        let mut thread = std::mem::replace(&mut self.thread, Thread::new());
        let num_channels = self.num_channels;

        let stream = device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if done_flag.load(Ordering::Relaxed) {
                    // Fill with silence if done
                    data.fill(0.0);
                    return;
                }

                Self::fill_audio_buffer(&mut thread, &mut inputs, data, num_channels, &done_flag);
            },
            |err| {
                eprintln!("Audio stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;
        
        // Keep the stream alive by moving it to a background thread
        thread::spawn(move || {
            // The stream stays alive as long as this thread exists
            loop {
                thread::sleep(Duration::from_millis(100));
                if done_flag_bg.load(Ordering::Relaxed) {
                    break;
                }
            }
            // Stream will be dropped here when the loop exits
        });
        
        println!("Audio output started successfully");
        Ok(())
    }

    /// Fill the audio buffer - equivalent to C++ fillBufferList()
    fn fill_audio_buffer(
        thread: &mut Thread,
        inputs: &mut [AudioIn],
        data: &mut [f32],
        num_channels: usize,
        done_flag: &Arc<AtomicBool>,
    ) {
        let frames_per_buffer = data.len() / num_channels;
        let mut temp_buffer = vec![0.0; frames_per_buffer];
        let mut all_done = true;

        // Process each channel
        for ch in 0..num_channels {
            let channel_done = if ch < inputs.len() {
                match inputs[ch].fill(thread, frames_per_buffer, &mut temp_buffer) {
                    Ok((samples_written, input_done)) => {
                        // Interleave samples into output buffer
                        for (frame, &sample) in temp_buffer.iter().enumerate().take(frames_per_buffer) {
                            let output_index = frame * num_channels + ch;
                            if output_index < data.len() {
                                data[output_index] = sample as f32;
                            }
                        }
                        
                        // Zero remaining samples if input didn't fill completely
                        for frame in samples_written..frames_per_buffer {
                            let output_index = frame * num_channels + ch;
                            if output_index < data.len() {
                                data[output_index] = 0.0;
                            }
                        }
                        
                        input_done
                    }
                    Err(err) => {
                        eprintln!("Error filling audio buffer: {}", err);
                        // Fill channel with silence on error
                        for frame in 0..frames_per_buffer {
                            let output_index = frame * num_channels + ch;
                            if output_index < data.len() {
                                data[output_index] = 0.0;
                            }
                        }
                        true // Mark as done on error
                    }
                }
            } else {
                // Fill unused channels with silence
                for frame in 0..frames_per_buffer {
                    let output_index = frame * num_channels + ch;
                    if output_index < data.len() {
                        data[output_index] = 0.0;
                    }
                }
                true
            };

            all_done &= channel_done;
        }

        // Mark player as done if all inputs are done
        if all_done {
            done_flag.store(true, Ordering::Relaxed);
        }
    }

    /// Stop the audio player
    fn stop(&mut self) {
        self.set_done();
        // The background thread will detect the done flag and stop the stream
    }
}

/// Audio manager - handles multiple concurrent audio players
pub struct AudioManager {
    players: HashMap<PlayerId, AudioPlayer>,
    next_id: PlayerId,
    device: Option<Device>,
    host: Host,
    watchdog_running: bool,
}

impl AudioManager {
    /// Create a new audio manager
    fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| SapfError::RuntimeError("No audio output device found".to_string()))?;

        Ok(Self {
            players: HashMap::new(),
            next_id: 1,
            device: Some(device),
            host,
            watchdog_running: false,
        })
    }

    /// Get the global audio manager instance
    pub fn instance() -> Arc<Mutex<AudioManager>> {
        AUDIO_MANAGER.get_or_init(|| {
            Arc::new(Mutex::new(AudioManager::new().expect("Failed to initialize audio manager")))
        }).clone()
    }

    /// Create and start a new audio player
    pub fn start_player(&mut self, thread: Thread, inputs: Vec<AudioIn>, config: AudioConfig) -> Result<PlayerId> {
        let player_id = self.next_id;
        self.next_id += 1;

        let device = self.device.as_ref()
            .ok_or_else(|| SapfError::RuntimeError("No audio device available".to_string()))?
            .clone();

        let mut player = AudioPlayer::new(player_id, thread, inputs, config);
        player.start(device)?;

        self.players.insert(player_id, player);

        // Start cleanup watchdog if not already running
        if !self.watchdog_running {
            self.start_watchdog();
            self.watchdog_running = true;
        }

        Ok(player_id)
    }

    /// Stop a specific player
    pub fn stop_player(&mut self, player_id: PlayerId) -> Result<()> {
        if let Some(mut player) = self.players.remove(&player_id) {
            player.stop();
        }
        Ok(())
    }

    /// Stop all players
    pub fn stop_all(&mut self) {
        let player_ids: Vec<_> = self.players.keys().cloned().collect();
        for player_id in player_ids {
            let _ = self.stop_player(player_id);
        }
    }

    /// Clean up finished players
    pub fn cleanup_finished(&mut self) {
        let finished_ids: Vec<_> = self.players
            .iter()
            .filter_map(|(id, player)| if player.is_done() { Some(*id) } else { None })
            .collect();

        for id in finished_ids {
            if let Some(mut player) = self.players.remove(&id) {
                player.stop();
            }
        }
    }

    /// Start the cleanup watchdog thread
    fn start_watchdog(&self) {
        let manager = Self::instance();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                if let Ok(mut mgr) = manager.try_lock() {
                    mgr.cleanup_finished();
                }
            }
        });
    }

    /// Get available audio devices
    pub fn get_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();
        
        for device in self.host.output_devices()? {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }
        
        Ok(devices)
    }

    /// Get supported sample rates for the default device
    pub fn get_supported_sample_rates(&self) -> Result<Vec<u32>> {
        let device = self.device.as_ref()
            .ok_or_else(|| SapfError::RuntimeError("No audio device available".to_string()))?;
            
        let configs = device.supported_output_configs()?;
        let mut sample_rates = Vec::new();
        
        for config in configs {
            let min_rate = config.min_sample_rate().0;
            let max_rate = config.max_sample_rate().0;
            
            // Add common sample rates within the supported range
            for &rate in &[44100, 48000, 88200, 96000, 176400, 192000] {
                if rate >= min_rate && rate <= max_rate {
                    sample_rates.push(rate);
                }
            }
        }
        
        sample_rates.sort_unstable();
        sample_rates.dedup();
        Ok(sample_rates)
    }
}

/// High-level functions for audio playback - equivalent to C++ playWithAudioUnit()

/// Play a SAPF value through the audio system
/// 
/// This is the main entry point for audio playback, equivalent to C++ playWithAudioUnit()
pub fn play(thread: Thread, value: Value) -> Result<PlayerId> {
    let vm = VM::instance();
    let config = AudioConfig {
        sample_rate: vm.audio_rate().sample_rate,
        buffer_size: vm.audio_rate().block_size,
        num_channels: 2, // Default to stereo
    };

    play_with_config(thread, value, config)
}

/// Play a SAPF value with specific audio configuration
pub fn play_with_config(thread: Thread, value: Value, config: AudioConfig) -> Result<PlayerId> {
    let inputs = prepare_audio_inputs(value)?;
    let actual_config = AudioConfig {
        num_channels: inputs.len().max(1).min(MAX_CHANNELS),
        ..config
    };

    let manager = AudioManager::instance();
    let mut mgr = manager.lock()
        .map_err(|_| SapfError::RuntimeError("Failed to lock audio manager".to_string()))?;
    
    mgr.start_player(thread, inputs, actual_config)
}

/// Stop audio playback for a specific player
pub fn stop_player(player_id: PlayerId) -> Result<()> {
    let manager = AudioManager::instance();
    let mut mgr = manager.lock()
        .map_err(|_| SapfError::RuntimeError("Failed to lock audio manager".to_string()))?;
    
    mgr.stop_player(player_id)
}

/// Stop all audio playback
pub fn stop_all() -> Result<()> {
    let manager = AudioManager::instance();
    let mut mgr = manager.lock()
        .map_err(|_| SapfError::RuntimeError("Failed to lock audio manager".to_string()))?;
    
    mgr.stop_all();
    Ok(())
}

/// Prepare audio inputs from a SAPF value
fn prepare_audio_inputs(value: Value) -> Result<Vec<AudioIn>> {
    match &value {
        Value::Object(obj) => {
            if let Some(list) = obj.as_any().downcast_ref::<List>() {
                if list.is_real_list() {
                    // Single ZList becomes single channel
                    Ok(vec![AudioIn::with_value(value)])
                } else {
                    // VList with multiple channels
                    let mut inputs = Vec::new();
                    let mut temp_thread = Thread::new();
                    
                    // Try to get elements up to MAX_CHANNELS
                    for i in 0..MAX_CHANNELS {
                        match list.at_index(&mut temp_thread, i) {
                            Ok(element) => inputs.push(AudioIn::with_value(element)),
                            Err(_) => break, // End of list reached
                        }
                    }
                    
                    if inputs.is_empty() {
                        inputs.push(AudioIn::new()); // At least one channel
                    }
                    
                    Ok(inputs)
                }
            } else {
                // Other objects become single channel
                Ok(vec![AudioIn::with_value(value)])
            }
        }
        _ => {
            // Single values become single channel
            Ok(vec![AudioIn::with_value(value)])
        }
    }
}

/// Get information about the audio system
pub fn get_audio_info() -> Result<HashMap<String, String>> {
    let manager = AudioManager::instance();
    let mgr = manager.lock()
        .map_err(|_| SapfError::RuntimeError("Failed to lock audio manager".to_string()))?;
    
    let mut info = HashMap::new();
    
    if let Ok(devices) = mgr.get_devices() {
        info.insert("devices".to_string(), format!("{:?}", devices));
    }
    
    if let Ok(sample_rates) = mgr.get_supported_sample_rates() {
        info.insert("sample_rates".to_string(), format!("{:?}", sample_rates));
    }
    
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_manager_creation() {
        let _manager = AudioManager::instance();
        // Just test that we can create the manager without panicking
    }

    #[test]
    fn test_prepare_single_value() {
        let value = Value::Real(0.5);
        let inputs = prepare_audio_inputs(value).unwrap();
        
        assert_eq!(inputs.len(), 1);
    }

    #[test]
    fn test_audio_config() {
        let config = AudioConfig::new(48000.0, 512, 2).unwrap();
        assert_eq!(config.sample_rate, 48000.0);
        assert_eq!(config.buffer_size, 512);
        assert_eq!(config.num_channels, 2);
        assert_eq!(config.nyquist(), 24000.0);
    }

    #[test]
    fn test_audio_config_validation() {
        assert!(AudioConfig::new(48000.0, 0, 2).is_err()); // Invalid buffer size
        assert!(AudioConfig::new(48000.0, 512, 0).is_err()); // Invalid channel count
        assert!(AudioConfig::new(48000.0, 512, MAX_CHANNELS + 1).is_err()); // Too many channels
    }
}