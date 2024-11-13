#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, gpio::Io, i2c::I2c, prelude::*};

use esp_alloc as _;

use esp_println::print;
#[cfg(feature = "selftest")]
use esp_println::println;

enum Tmp117Addrs {
    Gnd = 0x90 >> 1,
    Vplus = 0x92 >> 1,
    Sda = 0x94 >> 1,
    Scl = 0x96 >> 1,
}

#[allow(dead_code)]
enum Tmp117Registers {
    TempResult = 0x00,
    Configuration = 0x01, // default 0x0220
    THighLimit = 0x02,
    TLowLimit = 0x03,
    EEPROMUL = 0x04,
    EEPROM1 = 0x05,
    EEPROM2 = 0x06,
    Tempoffset = 0x07,
    EEPROM3 = 0x08,
    DeviceId = 0x0F, // default value 0x0117
}

#[derive(Debug, Copy, Clone)]
struct Tmp117 {
    pub addr: u8,
    temp: [u8; 2],
}

impl Tmp117 {
    pub fn new(addr: u8) -> Self {
        Tmp117 {
            addr,
            temp: [0xFF; 2],
        }
    }

    #[allow(dead_code)]
    pub fn convert_u16_f32(inp: u16) -> f32 {
        // convert some u16 to the TMP117/119 format f32 for the Celsius temperature
        let temp = (inp as f32) * 0.0078125_f32;
        return temp;
    }

    pub fn oneshot<T: _esp_hal_i2c_Instance>(
        &mut self,
        i2c: &mut esp_hal::i2c::I2c<T, esp_hal::Blocking>,
        delay: &mut esp_hal::delay::Delay,
    ) -> Option<u16> {
        // set up oneshot by writing 0b11 MOD[1:0]
        let cfg_os: [u8; 2] = [0x00, 0b00000011];

        #[cfg(feature = "selftest")]
        println!("addr: {:x}", self.addr);
        let res = i2c.write(self.addr, &cfg_os);
        match res {
            Ok(_) => {}
            Err(_) => {
                #[cfg(feature = "selftest")]
                println!("couldn't trigger sample");
                return Some(0_u16);
            }
        }
        // wait 1ms for TMP117 to sample
        delay.delay(1.millis());

        // read from device to struct
        let mut tb: [u8; 2] = [0; 2];
        let rb = [Tmp117Registers::TempResult as u8];
        let res = i2c.write_read(self.addr, &rb, &mut tb);

        match res {
            Ok(_) => {
                for a in 0..2 {
                    self.temp[a] = tb[a];
                }
                let i16tmp: u16 = (self.temp[0] as u16) << 8 | (self.temp[1] as u16);
                return Some(i16tmp);
            }
            Err(_) => {
                #[cfg(feature = "selftest")]
                println!("couldn't write read");
                return None;
            }
        }
    }
}

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut delay = Delay::new();

    esp_println::logger::init_logger_from_env();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // instantiate the i2c
    let mut i2c = I2c::new(peripherals.I2C0, io.pins.gpio19, io.pins.gpio20, 400.kHz());

    // create the vector of tmp117 units
    let mut ts_vec: [(Tmp117, bool); 4] = [
        (Tmp117::new(Tmp117Addrs::Gnd as u8), false),
        (Tmp117::new(Tmp117Addrs::Vplus as u8), false),
        (Tmp117::new(Tmp117Addrs::Sda as u8), false),
        (Tmp117::new(Tmp117Addrs::Scl as u8), false),
    ];

    // scan the bus and populate the vector with each responding TMP117
    use Tmp117Addrs::*;
    const POSSIBLE_ADDRESSES: [u8; 4] = [Gnd as u8, Vplus as u8, Sda as u8, Scl as u8];

    // check which devices are present on the bus (addresses 0x48-0x4b)
    for a in 0..4 {
        let address = POSSIBLE_ADDRESSES[a];
        let mut buf_dummy: [u8; 2] = [0; 2];
        let scan_buf: [u8; 1] = [Tmp117Registers::DeviceId as u8];
        match i2c.write_read(address, &scan_buf, &mut buf_dummy) {
            Ok(_) => {}
            Err(_e) => {
                #[cfg(feature = "selftest")]
                println!("FATAL {:?}: Initial I2C probe failed. Devices missing or I2C bus mis-terminated or shorted?", _e);
                panic!("Restarting.");
            }
        }

        match (buf_dummy[0], buf_dummy[1]) {
            (0x01, 0x17) => {
                // instantiate it
                ts_vec[a].1 = true;
                #[cfg(feature = "selftest")]
                println!("!! Device at {:x}", address);
            }
            _ => {
                #[cfg(feature = "selftest")]
                println!("No device at {:x}", address);
            }
        }
    }
    #[cfg(feature = "selftest")]
    log::info!("Entering main loop!");
    loop {
        for (mut sensor, val) in ts_vec {
            if val {
                let k = sensor.oneshot(&mut i2c, &mut delay).unwrap();
                print!("[{:02X}:{:04X}]", sensor.addr, k);
            }
        }
        print!("\r\n");
        delay.delay(500.millis());
    }
}
