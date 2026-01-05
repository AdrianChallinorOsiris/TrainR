use crate::error::{Result, TrainError};
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration, MissedTickBehavior};

/// LED subsets
pub const GREEN_LEDS: std::ops::RangeInclusive<u8> = 1..=6;
pub const AMBER_LEDS: std::ops::RangeInclusive<u8> = 7..=12;
pub const RED_LEDS: std::ops::RangeInclusive<u8> = 13..=24;

/// Total number of LEDs
pub const LED_COUNT: u8 = 24;

/// LED state for set_led_by_color function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedState {
    On,
    Off,
}

/// Maps LED number (1-24) to GPIO pin (4-27)
fn led_to_gpio_pin(led: u8) -> Result<u8> {
    if led < 1 || led > LED_COUNT {
        return Err(TrainError::InvalidParameter(
            format!("LED number must be between 1 and {}, got {}", LED_COUNT, led)
        ));
    }
    // LED 1 -> GPIO 4, LED 2 -> GPIO 5, ..., LED 24 -> GPIO 27
    Ok(led + 3)
}

/// LED controller using direct GPIO access
/// LEDs are numbered 1-24, mapped to GPIO pins 4-27
pub struct LedController {
    /// GPIO line handles for each LED (1-24)
    handles: Arc<RwLock<HashMap<u8, Arc<Mutex<LineHandle>>>>>,
    /// Track which LEDs are currently blinking and their task handles
    blink_handles: Arc<RwLock<HashMap<u8, tokio::task::JoinHandle<()>>>>,
}

impl LedController {
    /// Create a new LED controller
    /// Initializes all 24 LEDs on GPIO pins 4-27
    pub fn new() -> Result<Self> {
        let mut handles = HashMap::new();
        
        // Open GPIO chip (usually /dev/gpiochip0 on Raspberry Pi)
        let mut chip = Chip::new("/dev/gpiochip0")
            .map_err(|e| TrainError::GPIO(format!("Failed to open GPIO chip: {}", e)))?;
        
        // Initialize GPIO lines for LEDs 1-24 (GPIO pins 4-27)
        for led_num in 1..=LED_COUNT {
            let gpio_pin = led_to_gpio_pin(led_num)?;
            let line = chip.get_line(gpio_pin as u32)
                .map_err(|e| TrainError::GPIO(format!("Failed to get GPIO line {} for LED {}: {}", gpio_pin, led_num, e)))?;
            
            let handle = line.request(LineRequestFlags::OUTPUT, 0, "train-led")
                .map_err(|e| TrainError::GPIO(format!("Failed to request GPIO line {} for LED {}: {}", gpio_pin, led_num, e)))?;
            
            handles.insert(led_num, Arc::new(Mutex::new(handle)));
        }

        Ok(Self {
            handles: Arc::new(RwLock::new(handles)),
            blink_handles: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Turn on a specific LED (1-24)
    pub async fn on(&self, led: u8) -> Result<()> {
        // Cancel blinking if this LED is blinking
        self.cancel_blink(led).await?;
        
        let handles = self.handles.read().await;
        let handle = handles.get(&led)
            .ok_or_else(|| TrainError::InvalidParameter(format!("LED {} not found", led)))?;
        
        let handle_guard = handle.lock().await;
        handle_guard.set_value(1)
            .map_err(|e| TrainError::GPIO(format!("Failed to turn on LED {}: {}", led, e)))?;
        
        Ok(())
    }

    /// Turn off a specific LED (1-24)
    pub async fn off(&self, led: u8) -> Result<()> {
        // Cancel blinking if this LED is blinking
        self.cancel_blink(led).await?;
        
        let handles = self.handles.read().await;
        let handle = handles.get(&led)
            .ok_or_else(|| TrainError::InvalidParameter(format!("LED {} not found", led)))?;
        
        let handle_guard = handle.lock().await;
        handle_guard.set_value(0)
            .map_err(|e| TrainError::GPIO(format!("Failed to turn off LED {}: {}", led, e)))?;
        
        Ok(())
    }

    /// Blink a specific LED (1-24) with given frequency in milliseconds
    /// The LED will toggle on/off at the specified interval
    pub async fn blink(&self, led: u8, frequency_ms: u64) -> Result<()> {
        if frequency_ms == 0 {
            return Err(TrainError::InvalidParameter(
                "Blink frequency must be greater than 0".to_string()
            ));
        }

        // Cancel any existing blink for this LED
        self.cancel_blink(led).await?;

        let handles = Arc::clone(&self.handles);
        let blink_handles = Arc::clone(&self.blink_handles);
        
        // Get the handle for this LED
        let handles_read = handles.read().await;
        let handle = handles_read.get(&led)
            .ok_or_else(|| TrainError::InvalidParameter(format!("LED {} not found", led)))?
            .clone();
        drop(handles_read);

        // Spawn a task to handle blinking
        let handle_task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(frequency_ms));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            let mut state = false;

            loop {
                interval.tick().await;
                let handle_guard = handle.lock().await;
                state = !state;
                let _ = handle_guard.set_value(if state { 1 } else { 0 });
            }
        });

        // Store the handle
        let mut handles_write = blink_handles.write().await;
        handles_write.insert(led, handle_task);

        Ok(())
    }

    /// Cancel blinking for a specific LED
    async fn cancel_blink(&self, led: u8) -> Result<()> {
        let mut handles = self.blink_handles.write().await;
        if let Some(handle) = handles.remove(&led) {
            handle.abort();
        }
        Ok(())
    }

    /// Turn all LEDs off and cancel all blinking
    pub async fn all_off(&self) -> Result<()> {
        // Cancel all blinking first
        let mut handles = self.blink_handles.write().await;
        for handle in handles.values() {
            handle.abort();
        }
        handles.clear();
        drop(handles);

        // Turn off all LEDs
        let handles_read = self.handles.read().await;
        for (led, handle) in handles_read.iter() {
            let handle_guard = handle.lock().await;
            handle_guard.set_value(0)
                .map_err(|e| TrainError::GPIO(format!("Failed to turn off LED {}: {}", led, e)))?;
        }

        Ok(())
    }

    /// Get the number of LEDs
    pub fn count(&self) -> usize {
        LED_COUNT as usize
    }

    /// Check if an LED number is valid (1-24)
    pub fn is_valid_led(&self, led: u8) -> bool {
        led >= 1 && led <= LED_COUNT
    }

    /// Get the actual LED number from a color subset and position (1-based)
    /// 
    /// # Arguments
    /// * `subset` - The LED color range (GREEN_LEDS, AMBER_LEDS, or RED_LEDS)
    /// * `position` - Position within the subset (1-based, e.g., 1 = first LED in subset)
    /// 
    /// # Returns
    /// The actual LED number (1-24)
    /// 
    /// # Example
    /// ```
    /// // Get the 2nd red LED (LED 14)
    /// let led = get_led_from_subset(RED_LEDS, 2); // Returns 14
    /// ```
    fn get_led_from_subset(subset: std::ops::RangeInclusive<u8>, position: u8) -> Result<u8> {
        let start = *subset.start();
        let end = *subset.end();
        let count = end - start + 1;
        
        if position < 1 || position > count {
            return Err(TrainError::InvalidParameter(
                format!("Position {} is out of range for subset (1-{})", position, count)
            ));
        }
        
        // Position is 1-based, so subtract 1 to get 0-based offset
        Ok(start + position - 1)
    }

    /// Set LED state by color subset and position
    /// 
    /// # Arguments
    /// * `subset` - The LED color range (GREEN_LEDS, AMBER_LEDS, or RED_LEDS)
    /// * `position` - Position within the subset (1-based, e.g., 1 = first LED in subset)
    /// * `state` - LED state (On or Off)
    /// 
    /// # Example
    /// ```
    /// // Turn off the 2nd red LED (LED 14)
    /// controller.set_led_by_color(RED_LEDS, 2, LedState::Off).await?;
    /// 
    /// // Turn on the 3rd green LED (LED 3)
    /// controller.set_led_by_color(GREEN_LEDS, 3, LedState::On).await?;
    /// ```
    pub async fn set_led_by_color(
        &self,
        subset: std::ops::RangeInclusive<u8>,
        position: u8,
        state: LedState,
    ) -> Result<()> {
        let led = Self::get_led_from_subset(subset, position)?;
        match state {
            LedState::On => self.on(led).await,
            LedState::Off => self.off(led).await,
        }
    }

    /// Turn on a LED by color subset and position
    /// 
    /// # Example
    /// ```
    /// // Turn on the 2nd amber LED (LED 8)
    /// controller.green_on(2).await?;
    /// ```
    pub async fn green_on(&self, position: u8) -> Result<()> {
        self.set_led_by_color(GREEN_LEDS, position, LedState::On).await
    }

    /// Turn off a LED by color subset and position
    pub async fn green_off(&self, position: u8) -> Result<()> {
        self.set_led_by_color(GREEN_LEDS, position, LedState::Off).await
    }

    /// Turn on an amber LED by position
    pub async fn amber_on(&self, position: u8) -> Result<()> {
        self.set_led_by_color(AMBER_LEDS, position, LedState::On).await
    }

    /// Turn off an amber LED by position
    pub async fn amber_off(&self, position: u8) -> Result<()> {
        self.set_led_by_color(AMBER_LEDS, position, LedState::Off).await
    }

    /// Turn on a red LED by position
    pub async fn red_on(&self, position: u8) -> Result<()> {
        self.set_led_by_color(RED_LEDS, position, LedState::On).await
    }

    /// Turn off a red LED by position
    pub async fn red_off(&self, position: u8) -> Result<()> {
        self.set_led_by_color(RED_LEDS, position, LedState::Off).await
    }

    /// Blink a LED by color subset and position
    /// 
    /// # Example
    /// ```
    /// // Blink the 1st red LED (LED 13) at 500ms interval
    /// controller.blink_by_color(RED_LEDS, 1, 500).await?;
    /// ```
    pub async fn blink_by_color(
        &self,
        subset: std::ops::RangeInclusive<u8>,
        position: u8,
        frequency_ms: u64,
    ) -> Result<()> {
        let led = Self::get_led_from_subset(subset, position)?;
        self.blink(led, frequency_ms).await
    }
}
