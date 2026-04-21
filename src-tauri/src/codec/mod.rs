
pub fn f32_to_i16(samples: Vec<f32>) -> Vec<i16> {
    samples.into_iter().map(|s| {
        let clamped = s.clamp(-1.0, 1.0);
        (clamped * i16::MAX as f32) as i16
    }).collect()
}

pub fn i16_to_f32(samples: Vec<i16>) -> Vec<f32> {
    samples.into_iter().map(|s| s as f32 / i16::MAX as f32).collect()
}

const SEG_TABLE: [i32; 8] = [
    0x1F, 0x3F, 0x7F, 0xFF,
		0x1FF, 0x3FF, 0x7FF, 0xFFF
];

//Thanks to columbia university research project for C code inspiration
pub fn linear_to_alaw(mut samples: Vec<i16>) -> Vec<u8> {
    //mask is 0x55
    let mut out: Vec<u8> = Vec::with_capacity(samples.len());
    for mut s in samples.into_iter() {
        let mut mask: u8;
        if s >= 0 {
            mask = 0xD5;
        } else {
            mask = 0x55;
            s = -s - 1;
        }

        let mut seg = 0;
        let mut val = s as i32;

        while seg < 8 && val > SEG_TABLE[seg]{
            seg += 1;            
        }
        
        let aval: u8 = if seg >= 8 {
            0x7F
        } else {
            let mut a = (seg << 4) as u8;

            if seg < 2 {
                a |= ((s >> 1) & 0x0F) as u8;
            } else {
                a |= ((s >> seg) & 0x0F) as u8;
            }

            a
        };

        out.push(aval ^ mask);

    }

    out
}

pub fn alaw_to_linear(samples: Vec<u8>) -> Vec<i16> {
    let mut out: Vec<i16> = Vec::with_capacity(samples.len());
    
    for mut s in samples {
        s ^= 0x55;
        //0xf
        let mut t: i32 = ((s & 0xf) as i32) << 4;
        let seg = ((s & 0x70) >> 4) as i32;

        match seg {
            0 => {
                t += 8;
            }

            1 => {
                t += 0x108;
            }

            _ => {
                t += 0x108;
                t <<= seg - 1;
            }
        }

        let result = if (s & 0x80) != 0 {
            t
        } else {
            -t
        };

        out.push(result as i16);
    }
    
    out 
}
