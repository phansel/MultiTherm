// Read temperatures sequentially from serial
// Write array sequence to files (/tmp/tempdataline) as npyz
// Other programs can read npyz periodically as required

use serialport;
use std::io::BufRead;
use std::io::BufReader;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use std::vec::Vec;

// TODO: flush samples more intelligently
const TSAMPS_FLUSH_THRESH: usize = 1500;
const TSAMPS_FLUSH_TARGET: usize = 1025;
const TDIFF_CUTOFF_MS: u128 = 750; // fudge factor

fn main() {
    let mut temp_seq: [[u16; 3]; 16] = [[0; 3]; 16];

    let serialport = "/dev/ttyACM0";
    let bitrate = 115200;
    let data_filename = "/tmp/tempdataline";

    // timestamp durations in the past
    // should be reconstructed / interpolated from existing values
    let time_samples_dt = [
        0.0, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0, 1024.0, 2048.0, 4096.0,
        8192.0, 16384.0,
    ];

    // whether each of the samples in the line is valid (if not valid, set to 0)
    let mut time_samples_valid = [0_u8; 16];

    println!("Launching serial tempdata relay!");

    let mut tsamps: Vec<(std::time::Instant, [u16; 3])> = Vec::new();

    let serial_port = serialport::new(serialport, bitrate)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("Failed to open serial port.");

    let mut reader = BufReader::new(serial_port);

    // TODO: remove coupling to three sensors
    let addr_arr = [0x48_u8, 0x49_u8, 0x4A_u8];

    let mut buffer = "".to_string(); // enough bytes to read one line of temp samples
    loop {
        buffer = "".to_string();
        let read = reader.read_line(&mut buffer).unwrap();
        let current_time = Instant::now();
        let actual_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as u128;

        let x = string_to_temps(buffer);

        let y = map_tr_to_arr(x, addr_arr);

        if let Some(y) = y {
            tsamps.push((current_time, y));
            (tsamps, temp_seq) = process_samples(tsamps, &time_samples_dt, &mut time_samples_valid);
            match write_ts_data_file(data_filename.to_string(), actual_now, temp_seq) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Fault: {e}");
                }
            }
        }
    }
}

/*
Example output from the serial port:
[48:1606][49:1665][4a:1314]
[48:1606][49:1666][4a:1313]
[48:1606][49:1666][4a:1313]
temp at 0x48: 40.632813 C
temp at 0x49: 33.992188 C
temp at 0x4a: 24.695313 C

*/



#[derive(npyz::Serialize, npyz::Deserialize, npyz::AutoSerialize, Debug, PartialEq, Clone)]
pub struct SampleBlob {
    // TODO: remove coupling to three sensors
    arr: [[u16; 3]; 16],
    timestamp: [u32; 4],
}

// docs: https://docs.rs/npyz/latest/npyz/
pub fn write_ts_data_file(fname: String, ts: u128, arr: [[u16; 3]; 16]) -> std::io::Result<()> {
    // TODO: remove coupling to three sensors
    let mut out_buf: Vec<SampleBlob> = vec![];
    let mut structs = vec![];

    let tsu32 = [
        ts as u32,
        (ts >> 32) as u32,
        (ts >> 64) as u32,
        (ts >> 96) as u32,
    ];

    let calBlob = SampleBlob {
        arr: arr,
        timestamp: tsu32,
    };

    structs.push(calBlob);

    npyz::to_file_1d(fname.to_string(), structs)?;

    Ok(())
}

pub fn process_samples(
    mut tsamps: Vec<(std::time::Instant, [u16; 3])>,
    sample_deltas: &[f32; 16],
    mut mask_valid: &mut [u8; 16],
) -> (Vec<(std::time::Instant, [u16; 3])>, [[u16; 3]; 16]) {
    // TODO: remove coupling to three sensors
    let mut temp_hist_arr = [[0xFFFF_u16; 3]; 16];

    let time_now = Instant::now();

    // sample the data points and interpolate, and also cull extra points if over 1000
    if tsamps.len() > TSAMPS_FLUSH_THRESH {
        let mut tsamps_new: Vec<(std::time::Instant, [u16; 3])> = Vec::new();
        let todrain = TSAMPS_FLUSH_TARGET;

        for i in 0..todrain {
            tsamps_new.push(tsamps[i + todrain - 1]);
        }
        tsamps = tsamps_new;
    }

    // wipe out the mask if we don't have samples back then
    let oldest_delta = ((time_now - tsamps[0].0).as_millis() as f32) / 1000.0;
    let mut n_valid = 0;
    for i in 0..mask_valid.len() {
        if sample_deltas[i] > oldest_delta {
            mask_valid[i] = false as u8;
        } else {
            mask_valid[i] = true as u8;
            n_valid += 1;
        }
    }

    // sample tsamps at sample_deltas offsets from time_now
    // is this inefficient? why not
    for i in 0..n_valid {
        let target_tdiff = (sample_deltas[i] * 1000.0) as u128;
        for (time, tempvals) in &tsamps {
            if ((time_now - *time).as_millis() - target_tdiff) < TDIFF_CUTOFF_MS {
                temp_hist_arr[i] = *tempvals;
                break;
            }
        }
        if temp_hist_arr[i] == [0xFFFF_u16; 3] {
            // todo: if we didn't get it with nearest neighbor, try interp
            // println!("Didn't get it on n{}", i);
        }
    }

    return (tsamps, temp_hist_arr);
}

pub fn map_tr_to_arr(temps: [(u8, u16); 3], addr_arr: [u8; 3]) -> Option<[u16; 3]> {
    // TODO: remove coupling to three sensors
    let mut temps2 = [0_u16; 3];
    let mut incr = 0;
    for addr in addr_arr {
        for temp in temps {
            if temp.0 == addr {
                temps2[incr] = temp.1;
                incr += 1;
            }
        }
    }
    if incr == 3 {
        return Some(temps2);
    } else {
        return None;
    }
}

pub fn string_to_temps(instr: String) -> [(u8, u16); 3] {
    // TODO: remove coupling to three sensors
    let mut outdata = [(0_u8, 0_u16); 3];

    let instr = instr.trim();

    match instr.chars().nth(0) {
        Some('[') => {}
        _ => {
            println!("Not a valid string! Should probably error.");
            return [(0_u8, 0_u16); 3];
        }
    };

    match instr.chars().nth(26) {
        Some(']') => {}
        _ => {
            println!("Not a valid string! Should probably error.");
            return [(0_u8, 0_u16); 3];
        }
    };

    // [48:1606][49:1666][4a:1313]

    let (addr1, data1) = (&instr[1..3], &instr[4..8]);
    let (addr2, data2) = (&instr[10..12], &instr[13..17]);
    let (addr3, data3) = (&instr[19..21], &instr[22..26]);

    // println!("vals: {}_{}, {}_{}, {}_{}", addr1, data1, addr2, data2, addr3, data3);

    // now put it into outdata
    // convert each str to a u16 or u8
    outdata[0] = (
        u8::from_str_radix(addr1, 16).unwrap(),
        u16::from_str_radix(data1, 16).unwrap(),
    );
    outdata[1] = (
        u8::from_str_radix(addr2, 16).unwrap(),
        u16::from_str_radix(data2, 16).unwrap(),
    );
    outdata[2] = (
        u8::from_str_radix(addr3, 16).unwrap(),
        u16::from_str_radix(data3, 16).unwrap(),
    );

    // println!("outdata: {:x?}", outdata);
    return outdata;
}
