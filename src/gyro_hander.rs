// src/input.rs
use bevy::prelude::*;

#[cfg(feature = "raspberry_pi")]
use rppal::i2c::I2c;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>();

        #[cfg(feature = "raspberry_pi")]
        app.add_systems(Startup, setup_gyro)
           .add_systems(Update, update_gyro);

        #[cfg(feature = "keyboard_controls")]
        app.add_systems(Update, update_keyboard);
    }
}

// Common input state that both control schemes will update
#[derive(Resource, Default)]
pub struct InputState {
    pub horizontal_movement: f32,  // -1.0 to 1.0
    pub shoot_requested: bool,
}

#[cfg(feature = "raspberry_pi")]
mod gyro_controls {
    use super::*;
    use std::error::Error;

    const MPU6050_ADDR: u16 = 0x68;
    const GYRO_CONFIG: u8 = 0x1B;
    const GYRO_XOUT_H: u8 = 0x43;
    const PWR_MGMT_1: u8 = 0x6B;
    const TILT_THRESHOLD: f32 = 15.0;

    #[derive(Resource)]
    struct GyroDevice {
        i2c: Option<I2c>,
    }

    impl Default for GyroDevice {
        fn default() -> Self {
            Self { i2c: None }
        }
    }

    pub(super) fn setup_gyro(mut commands: Commands) {
        match I2c::new() {
            Ok(mut i2c) => {
                if let Err(e) = initialize_mpu6050(&mut i2c) {
                    error!("Failed to initialize MPU6050: {}", e);
                    return;
                }
                commands.insert_resource(GyroDevice { i2c: Some(i2c) });
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

    pub(super) fn update_gyro(
        gyro: Option<ResMut<GyroDevice>>,
        mut input_state: ResMut<InputState>
    ) {
        if let Some(mut gyro) = gyro {
            if let Some(i2c) = &mut gyro.i2c {
                if let Ok((x_tilt, y_tilt)) = read_gyro_data(i2c) {
                    let x_normalized = (x_tilt as f32 / 131.0 / 90.0).clamp(-1.0, 1.0);
                    let y_normalized = y_tilt as f32 / 131.0;
                    
                    input_state.horizontal_movement = x_normalized;
                    input_state.shoot_requested = y_normalized > TILT_THRESHOLD;
                }
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