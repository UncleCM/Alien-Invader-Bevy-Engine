use std::ops::Add;

// src/input.rs
use bevy::prelude::*;

#[cfg(feature = "raspberry_pi")]
use rppal::gpio::{Gpio, InputPin, Level};
#[cfg(feature = "raspberry_pi")]
use rppal::i2c::I2c;    

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>();

        #[cfg(feature = "raspberry_pi")]
            app.add_systems(Startup, setup_input)
            .add_systems(Update, update_input);

        #[cfg(feature = "keyboard_controls")]
        app.add_systems(Update, update_keyboard);
    }
}

// Common input state that both control schemes will update
#[derive(Resource, Default)]
pub struct InputState {
    pub horizontal_movement: f32,  // -1.0 to 1.0
    pub shoot_requested: bool,
    pub shoot_button_pressed: bool,
}

#[cfg(feature = "raspberry_pi")]
mod input_controls {
    use super::*;
    use std::error::Error;

    const MPU6050_ADDR: u16 = 0x68;
    const GYRO_CONFIG: u8 = 0x1B;
    const GYRO_XOUT_H: u8 = 0x43;
    const PWR_MGMT_1: u8 = 0x6B;
    const TILT_THRESHOLD: f32 = 15.0;
    const SHOOT_BUTTON_PIN: u8 = 17; // GPIO pin number for the shoot button

    #[derive(Resource)]
    struct InputDevices {
        i2c: Option<I2c>,
        button: Option<InputPin>,
    }

    impl Default for InputDevices {
        fn default() -> Self {
            Self { i2c: None, button: None }
        }
    }

    pub(super) fn setup_input(mut commands: Commands) {
        match I2c::new() {
            Ok(mut i2c) => {
                if let Err(e) = initialize_mpu6050(&mut i2c) {
                    error!("Failed to initialize MPU6050: {}", e);
                    return;
                }

                let mut gpio = Gpio::new().unwrap();
                let mut button = gpio.get(SHOOT_BUTTON_PIN).unwrap().into_input_pulled_up();

                commands.insert_resource(InputDevices { i2c: Some(i2c), button: Some(button) });
            }
            Err(e) => {
                error!("Failed to open I2C connection: {}", e);
            }
        }
    }

    fn initialize_mpu6050(i2c: &mut I2c) -> Result<(), Box<dyn Error>> {
        i2c.write(MPU6050_ADDR, &[PWR_MGMT_1, 0x00])?;
        i2c.write(MPU6050_ADDR, &[GYRO_CONFIG, 0x00])?;
        Ok(())
    }

    pub(super) fn update_input(
        input_devices: Option<ResMut<InputDevices>>,
        mut input_state: ResMut<InputState>
    ) {
        if let Some(mut input_devices) = input_devices {
            if let Some(i2c) = &mut input_devices.i2c {
                if let Ok((x_tilt, y_tilt)) = read_gyro_data(i2c) {
                    let x_normalized = (x_tilt as f32 / 131.0 / 90.0).clamp(-1.0, 1.0);
                    let y_normalized = y_tilt as f32 / 131.0;
                    
                    input_state.horizontal_movement = x_normalized;
                    input_state.shoot_requested = y_normalized > TILT_THRESHOLD;
                }
            }

            if let Some(button) = &mut input_devices.button {
                input_state.shoot_button_pressed = button.is_low();
            }
        }
    }

    fn read_gyro_data(i2c: &mut I2c) -> Result<(i16, i16), Box<dyn Error>> {
        let mut buf = [0u8; 4];
        i2c.write_read(MPU6050_ADDR, &[GYRO_XOUT_H], &mut buf)?;
        let gyro_x = (buf[0] as i16) << 8 | buf[1] as i16;
        let gyro_y = (buf[2] as i16) << 8 | buf[3] as i16;
        Ok((gyro_x, gyro_y))
    }
}

#[cfg(feature = "keyboard_controls")]
fn update_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut input_state: ResMut<InputState>
) {
    let mut horizontal = 0.0;
    
    if keys.pressed(KeyCode::KeyA) {
        horizontal -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        horizontal += 1.0;
    }

    input_state.horizontal_movement = horizontal;
    input_state.shoot_requested = keys.pressed(KeyCode::Space);
}