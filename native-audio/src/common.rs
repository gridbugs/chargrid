use rodio::DeviceTrait;

pub fn output_device() -> Option<rodio::Device> {
    let maybe_device = rodio::default_output_device();
    if let Some(device) = maybe_device.as_ref() {
        match device.name() {
            Ok(device_name) => log::info!("Using audio device: {}", device_name),
            Err(error) => log::warn!("Using audio device with unknown name (error: {})", error),
        }
    } else {
        log::warn!("Unable to find audio device. Audio will be disabled.");
    }
    maybe_device
}
