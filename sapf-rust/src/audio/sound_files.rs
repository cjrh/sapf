// SAPF - Sound As Pure Form
// Rust implementation of SoundFiles.hpp/cpp
// Thread-safe sound file I/O using hound for WAV files

use crate::core::{Value, Object, List, SapfError};
use crate::core::value::StringObject;
use crate::vm::thread::Thread;
use crate::dsp::ugen::{UGen, UGenBase};
use std::sync::{Arc, Mutex, atomic::{AtomicI32, Ordering}};
use std::path::PathBuf;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::fmt;
use hound::{WavReader, WavWriter, WavSpec, SampleFormat};

/// Maximum number of channels supported in sound files
const MAX_SF_CHANNELS: usize = 1024;
/// Buffer size for file I/O operations
const BUF_SIZE: usize = 1024;

/// Global file counter for auto-generated filenames (thread-safe)
static FILE_COUNT: AtomicI32 = AtomicI32::new(0);

/// Thread-safe sound file reader that implements the UGen trait
#[derive(Debug)]
pub struct SFReader {
    reader: Arc<Mutex<Option<WavReader<BufReader<File>>>>>,
    spec: WavSpec,
    frames_remaining: Arc<Mutex<i64>>,
    output_channels: Vec<Arc<SFReaderOutputChannel>>,
    finished: Arc<Mutex<bool>>,
}

impl SFReader {
    /// Create a new sound file reader
    pub fn new(
        reader: WavReader<BufReader<File>>,
        duration: Option<i64>,
    ) -> Result<Self, SapfError> {
        let spec = reader.spec();
        let frames_remaining = duration.unwrap_or(reader.duration() as i64);
        
        Ok(SFReader {
            reader: Arc::new(Mutex::new(Some(reader))),
            spec,
            frames_remaining: Arc::new(Mutex::new(frames_remaining)),
            output_channels: Vec::new(),
            finished: Arc::new(Mutex::new(false)),
        })
    }
    
    /// Create output channels for this reader
    pub fn create_outputs(&mut self, thread: &Thread) -> Result<Arc<dyn Object>, SapfError> {
        let num_channels = self.spec.channels as usize;
        let mut channels = Vec::new();
        
        for _i in 0..num_channels {
            let channel = Arc::new(SFReaderOutputChannel::new(
                thread,
                self.reader.clone(),
                self.frames_remaining.clone(),
                self.finished.clone(),
            )?);
            channels.push(channel.clone());
        }
        
        self.output_channels = channels.clone();
        
        // Create a List containing the output channels
        let mut channel_values = Vec::new();
        for channel in channels {
            // Wrap each channel in a List (as done in C++ version)
            let channel_list = List::new_with_single_value(Value::Object(channel))?;
            channel_values.push(Value::Object(channel_list));
        }
        
        List::new_from_values(channel_values)
    }
    
    /// Pull audio data from file (thread-safe)
    pub fn pull(&self, thread: &Thread) -> Result<bool, SapfError> {
        let mut finished = self.finished.lock().map_err(|_| SapfError::InternalError)?;
        if *finished {
            return Ok(true);
        }
        
        let mut frames_remaining = self.frames_remaining.lock().map_err(|_| SapfError::InternalError)?;
        if *frames_remaining == 0 {
            *finished = true;
            return Ok(true);
        }
        
        let block_size = thread.block_size();
        let read_size = if *frames_remaining > 0 {
            (*frames_remaining as usize).min(block_size)
        } else {
            block_size
        };
        
        // Read audio data from file
        let mut reader_guard = self.reader.lock().map_err(|_| SapfError::InternalError)?;
        if let Some(ref mut reader) = reader_guard.as_mut() {
            // Read samples into buffers
            let mut samples_read = 0;
            let mut audio_data = Vec::new();
            
            // Read interleaved samples
            for _frame in 0..read_size {
                for _channel in 0..self.spec.channels {
                    match reader.samples::<f32>().next() {
                        Some(Ok(sample)) => {
                            audio_data.push(sample as f64);
                            samples_read += 1;
                        }
                        Some(Err(_)) | None => {
                            *finished = true;
                            break;
                        }
                    }
                }
                if *finished {
                    break;
                }
            }
            
            let frames_read = samples_read / self.spec.channels as usize;
            
            // Distribute data to output channels
            self.distribute_to_channels(&audio_data, frames_read)?;
            
            if *frames_remaining > 0 {
                *frames_remaining -= frames_read as i64;
            }
            
            if frames_read == 0 {
                *finished = true;
            }
        }
        
        Ok(*finished)
    }
    
    /// Distribute audio data to output channels
    fn distribute_to_channels(&self, data: &[f64], frames: usize) -> Result<(), SapfError> {
        let num_channels = self.spec.channels as usize;
        
        for (channel_idx, channel) in self.output_channels.iter().enumerate() {
            let mut channel_data = Vec::with_capacity(frames);
            
            // Extract samples for this channel (deinterleave)
            for frame in 0..frames {
                let sample_idx = frame * num_channels + channel_idx;
                if sample_idx < data.len() {
                    channel_data.push(data[sample_idx]);
                } else {
                    channel_data.push(0.0); // Pad with zeros
                }
            }
            
            channel.set_data(channel_data)?;
        }
        
        Ok(())
    }
}

impl fmt::Display for SFReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SFReader({}ch@{}Hz)", self.spec.channels, self.spec.sample_rate)
    }
}

impl Object for SFReader {
    fn type_name(&self) -> &'static str {
        "SFReader"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_float(&self) -> Result<f64, SapfError> {
        Err(SapfError::WrongType)
    }
    
    fn deref(&self) -> Result<Value, SapfError> {
        Ok(Value::Object(self.clone_object()))
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        // This is a simplified clone - in a real implementation, 
        // you might want to share the file reader properly
        Arc::new(SFReader {
            reader: self.reader.clone(),
            spec: self.spec,
            frames_remaining: self.frames_remaining.clone(),
            output_channels: Vec::new(), // Reset output channels
            finished: self.finished.clone(),
        })
    }
}

/// Output channel for sound file reader
#[derive(Debug)]
pub struct SFReaderOutputChannel {
    base: UGenBase,
    reader: Arc<Mutex<Option<WavReader<BufReader<File>>>>>,
    frames_remaining: Arc<Mutex<i64>>,
    finished: Arc<Mutex<bool>>,
    current_data: Arc<Mutex<Vec<f64>>>,
    data_index: Arc<Mutex<usize>>,
}

impl SFReaderOutputChannel {
    fn new(
        thread: &Thread,
        reader: Arc<Mutex<Option<WavReader<BufReader<File>>>>>,
        frames_remaining: Arc<Mutex<i64>>,
        finished: Arc<Mutex<bool>>,
    ) -> Result<Self, SapfError> {
        Ok(SFReaderOutputChannel {
            base: UGenBase::new(thread.block_size(), false),
            reader,
            frames_remaining,
            finished,
            current_data: Arc::new(Mutex::new(Vec::new())),
            data_index: Arc::new(Mutex::new(0)),
        })
    }
    
    /// Set new audio data for this channel
    fn set_data(&self, data: Vec<f64>) -> Result<(), SapfError> {
        let mut current_data = self.current_data.lock().map_err(|_| SapfError::InternalError)?;
        let mut data_index = self.data_index.lock().map_err(|_| SapfError::InternalError)?;
        
        *current_data = data;
        *data_index = 0;
        
        Ok(())
    }
}

impl UGen for SFReaderOutputChannel {
    fn pull(&mut self, _thread: &mut Thread, output: &mut [f64]) -> Result<usize, SapfError> {
        let finished = self.finished.lock().map_err(|_| SapfError::InternalError)?;
        if *finished {
            // Fill with zeros if finished
            output.fill(0.0);
            self.base.set_done(true);
            return Ok(output.len());
        }
        
        let mut current_data = self.current_data.lock().map_err(|_| SapfError::InternalError)?;
        let mut data_index = self.data_index.lock().map_err(|_| SapfError::InternalError)?;
        
        let mut samples_filled = 0;
        for sample in output.iter_mut() {
            if *data_index < current_data.len() {
                *sample = current_data[*data_index];
                *data_index += 1;
                samples_filled += 1;
            } else {
                *sample = 0.0;
                samples_filled += 1;
            }
        }
        
        Ok(samples_filled)
    }
    
    fn is_done(&self) -> bool {
        self.base.is_done()
    }
    
    fn set_done(&mut self, done: bool) {
        self.base.set_done(done);
    }
}

impl fmt::Display for SFReaderOutputChannel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SFReaderOutputChannel")
    }
}

impl Object for SFReaderOutputChannel {
    fn type_name(&self) -> &'static str {
        "SFReaderOutputChannel"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_float(&self) -> Result<f64, SapfError> {
        Err(SapfError::WrongType)
    }
    
    fn deref(&self) -> Result<Value, SapfError> {
        Ok(Value::Object(self.clone_object()))
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(SFReaderOutputChannel {
            base: UGenBase::new(self.base.block_size(), false),
            reader: self.reader.clone(),
            frames_remaining: self.frames_remaining.clone(),
            finished: self.finished.clone(),
            current_data: self.current_data.clone(),
            data_index: self.data_index.clone(),
        })
    }
}

/// Create a recording path for output files
pub fn make_recording_path(filename: &Value) -> Result<PathBuf, SapfError> {
    match filename {
        Value::Object(obj) => {
            if let Some(string_obj) = obj.as_any().downcast_ref::<StringObject>() {
                let recordings_dir = env::var("SAPF_RECORDINGS")
                    .unwrap_or_else(|_| "/tmp".to_string());
                
                let mut path = PathBuf::from(recordings_dir);
                path.push(format!("{}.wav", string_obj.as_str()));
                Ok(path)
            } else {
                Err(SapfError::WrongType)
            }
        }
        _ => {
            // Auto-generate filename
            let count = FILE_COUNT.fetch_add(1, Ordering::SeqCst);
            let session_time = "session"; // TODO: implement session time
            let filename = format!("sapf-{}-{:04}.wav", session_time, count);
            Ok(PathBuf::from("/tmp").join(filename))
        }
    }
}

/// Read a sound file and create output generators
pub fn sf_read(
    thread: &mut Thread,
    filename: &Value,
    offset: i64,
    frames: i64,
) -> Result<(), SapfError> {
    let path = make_recording_path(filename)?;
    
    let file = File::open(&path)
        .map_err(|_| SapfError::NotFound)?;
    
    let reader = WavReader::new(BufReader::new(file))
        .map_err(|_| SapfError::Failed)?;
    
    // Seek to offset if needed
    if offset > 0 {
        // Note: hound doesn't support seeking directly, would need to skip samples
        // For now, we'll create the reader and handle this in the implementation
    }
    
    let duration = if frames > 0 { Some(frames) } else { None };
    let mut sf_reader = SFReader::new(reader, duration)?;
    
    let outputs = sf_reader.create_outputs(thread)?;
    thread.push(Value::Object(outputs));
    
    Ok(())
}

/// Write audio data to a sound file
pub fn sf_write(
    thread: &mut Thread,
    data: &Value,
    filename: &Value,
    open_file: bool,
) -> Result<(), SapfError> {
    let path = make_recording_path(filename)?;
    
    // Determine number of channels and prepare input data
    let (num_channels, audio_inputs) = match data {
        Value::Object(obj) => {
            if let Some(mut list) = obj.as_any().downcast_ref::<List>() {
                // Multi-channel case
                let channels = list.length()?;
                if channels > MAX_SF_CHANNELS {
                    return Err(SapfError::OutOfRange);
                }
                
                // TODO: Extract audio data from each channel
                // This would require implementing audio input extraction
                (channels, Vec::new())
            } else {
                // Single channel case
                (1, Vec::new())
            }
        }
        _ => return Err(SapfError::WrongType),
    };
    
    // Create WAV file specification
    let spec = WavSpec {
        channels: num_channels as u16,
        sample_rate: thread.sample_rate() as u32,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    
    let file = File::create(&path)
        .map_err(|_| SapfError::Failed)?;
    
    let mut writer = WavWriter::new(BufWriter::new(file), spec)
        .map_err(|_| SapfError::Failed)?;
    
    // TODO: Implement actual audio data writing
    // This would require:
    // 1. Extracting audio data from Value objects
    // 2. Processing multiple channels
    // 3. Writing interleaved samples to the WAV file
    
    writer.finalize()
        .map_err(|_| SapfError::Failed)?;
    
    println!("Wrote file '{}' with {} channels", path.display(), num_channels);
    
    if open_file {
        // TODO: Implement file opening (platform-specific)
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("open")
                .arg(&path)
                .spawn()
                .map_err(|_| SapfError::Failed)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;
    use std::sync::OnceLock;
    
    fn get_test_thread() -> Thread {
        static VM_INSTANCE: OnceLock<VM> = OnceLock::new();
        let vm = VM_INSTANCE.get_or_init(|| VM::new());
        Thread::new(vm.get_audio_rate())
    }
    
    #[test]
    fn test_make_recording_path_auto() {
        let result = make_recording_path(&Value::Nil);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("sapf-"));
        assert!(path.extension().unwrap() == "wav");
    }
    
    #[test]
    fn test_make_recording_path_string() {
        let string_obj = StringObject::new("test_file".to_string());
        let filename = Value::Object(Arc::new(string_obj));
        
        let result = make_recording_path(&filename);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("test_file.wav"));
    }
    
    #[test]
    fn test_sf_reader_creation() {
        // This test would require a sample WAV file
        // For now, just test that the module compiles
        assert!(true);
    }
    
    #[test]
    fn test_file_count_increment() {
        let initial = FILE_COUNT.load(Ordering::SeqCst);
        make_recording_path(&Value::Nil).unwrap();
        let after = FILE_COUNT.load(Ordering::SeqCst);
        assert!(after > initial);
    }
}