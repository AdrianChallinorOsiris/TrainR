use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrainError {
    #[error("Hardware interface error: {0}")]
    Hardware(String),

    #[error("I2C communication error: {0}")]
    I2C(String),

    #[error("GPIO error: {0}")]
    GPIO(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Device not found or not responding")]
    DeviceNotFound,

    #[error("Operation not supported")]
    NotSupported,
}

pub type Result<T> = std::result::Result<T, TrainError>;

impl From<gpio_cdev::Error> for TrainError {
    fn from(err: gpio_cdev::Error) -> Self {
        TrainError::GPIO(err.to_string())
    }
}
