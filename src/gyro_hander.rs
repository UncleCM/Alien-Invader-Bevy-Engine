use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing InputPlugin");
        app.init_resource::<InputState>();

        #[cfg(feature = "raspberry_pi")]
        app.add_systems(Startup, input_controls::setup_input)
            .add_systems(Update, input_controls::update_input);

        #[cfg(feature = "keyboard_controls")]
        app.add_systems(Update, update_keyboard);
    }
}

#[derive(Resource, Default)]
pub struct InputState {
    pub horizontal_movement: f32,
    pub shoot_requested: bool,
    pub shoot_button_pressed: bool,
}

#[cfg(feature = "raspberry_pi")]
mod input_controls {
    use super::*;
    use std::error::Error;
    use std::sync::Mutex;
    use once_cell::sync::OnceCell;

    const MPU6050_ADDR: u16 = 0x68;
    const GYRO_CONFIG: u8 = 0x1B;
    const GYRO_XOUT_H: u8 = 0x43;
    const PWR_MGMT_1: u8 = 0x6B;
    const TILT_THRESHOLD: f32 = 15.0;
    const SHOOT_BUTTON_PIN: u8 = 17;

    static I2C_DEVICE: OnceCell<Mutex<rppal::i2c::I2c>> = OnceCell::new();
    static BUTTON: OnceCell<Mutex<rppal::gpio::InputPin>> = OnceCell::new();

    #[derive(Resource)]
    pub struct InputInitialized(bool);

    impl Default for InputInitialized {
        fn default() -> Self {
            Self(false)
        }
    }

    fn initialize_mpu6050(i2c: &mut rppal::i2c::I2c) -> Result<(), Box<dyn Error>> {
        info!("Initializing MPU6050...");
        
        // Add delay before initialization
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        info!("Setting slave address to 0x{:02X}", MPU6050_ADDR);
        i2c.set_slave_address(MPU6050_ADDR)?;
        
        info!("Waking up device...");
        i2c.block_write(PWR_MGMT_1, &[0x00])?;
        
        // Add small delay after wake-up
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        info!("Configuring gyroscope...");
        i2c.block_write(GYRO_CONFIG, &[0x00])?;
        
        info!("MPU6050 initialization complete");
        Ok(())
    }

    pub fn setup_input(mut commands: Commands) {
        info!("Starting input setup...");
        
        match rppal::i2c::I2c::new() {
            Ok(mut i2c) => {
                info!("I2C connection established");
                
                match initialize_mpu6050(&mut i2c) {
                    Ok(_) => {
                        if I2C_DEVICE.set(Mutex::new(i2c)).is_err() {
                            error!("Failed to initialize I2C device global");
                            return;
                        }
                        info!("I2C device initialized successfully");
                    }
                    Err(e) => {
                        error!("Failed to initialize MPU6050: {}", e);
                        return;
                    }
                }

                if let Ok(mut gpio) = rppal::gpio::Gpio::new() {
                    info!("GPIO connection established");
                    if let Ok(button) = gpio.get(SHOOT_BUTTON_PIN) {
                        let button = button.into_input_pullup();
                        if BUTTON.set(Mutex::new(button)).is_err() {
                            error!("Failed to initialize button global");
                            return;
                        }
                        info!("Button initialized successfully");
                    }
                }

                commands.insert_resource(InputInitialized(true));
                info!("Input setup completed successfully");
            }
            Err(e) => {
                error!("Failed to open I2C connection: {}", e);
            }
        }
    }

    fn read_gyro_data() -> Result<(i16, i16), Box<dyn Error>> {
        let mut i2c = I2C_DEVICE.get()
            .ok_or("I2C device not initialized")?
            .lock()
            .map_err(|_| "Failed to lock I2C device")?;

        let mut buf = [0u8; 4];
        i2c.set_slave_address(MPU6050_ADDR)?;
        i2c.block_read(GYRO_XOUT_H, &mut buf)?;
        
        let gyro_x = (buf[0] as i16) << 8 | buf[1] as i16;
        let gyro_y = (buf[2] as i16) << 8 | buf[3] as i16;
        Ok((gyro_x, gyro_y))
    }

    pub fn update_input(
        initialized: Option<Res<InputInitialized>>,
        mut input_state: ResMut<InputState>
    ) {
        if initialized.map_or(false, |init| init.0) {
            match read_gyro_data() {
                Ok((x_tilt, y_tilt)) => {
                    let x_normalized = (x_tilt as f32 / 131.0 / 90.0).clamp(-1.0, 1.0);
                    let y_normalized = y_tilt as f32 / 131.0;
                    
                    input_state.horizontal_movement = x_normalized;
                    input_state.shoot_requested = y_normalized > TILT_THRESHOLD;
                }
                Err(e) => {
                    error!("Failed to read gyro data: {}", e);
                }
            }

            if let Some(button) = BUTTON.get() {
                if let Ok(button) = button.lock() {
                    input_state.shoot_button_pressed = button.is_low();
                }
            }
        }
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
