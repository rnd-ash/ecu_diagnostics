
// Since presentations are used for both Service01 and Service02,
// They will be kept in this file

use crate::{DiagServerResult, DiagError};

pub enum ObdValueType {
    Encoded(String),
    Value {
        value: f32,
        unit: Option<String>
    }
}

pub struct DataPresentation {
    description: String,
    value: ObdValueType,
}

impl DataPresentation {
    pub (crate) fn from_number(desc: &str, value: f32, unit: Option<String>) -> Self {
        Self {
            description: desc.to_string(),
            value: ObdValueType::Value { value, unit }
        }
    }

    pub (crate) fn from_enum(desc: &str, value: String) -> Self {
        Self {
            description: desc.to_string(),
            value: ObdValueType::Encoded(value)
        }
    }
}

struct DataPid {
    pid: u8,
    required_size: u8,
    name: &'static str,
    desc: &'static str,
    f: fn(&[u8]) -> DiagServerResult<Vec<DataPresentation>>
}

impl DataPid {
    pub fn parse(&self, data: &[u8]) -> DiagServerResult<Vec<DataPresentation>> {
        if data.len() != self.required_size as usize {
            Err(DiagError::InvalidResponseLength)
        } else {
            (self.f)(data)
        }
    }
}

pub (crate) fn get_obd_supported_pids(start: u8, src: &[u8]) -> Vec<u8> {
    let mut res = Vec::new();
    let mut s = start;
    for byte in src {
        let mut mask: u8 = 0b10000000;
        for _ in 0..8 {
            if byte & mask != 0x00 {
                res.push(s);
            }
            mask = mask >> 1;
            s+=1;
        }
    }
    res
}

const OBD_PIDS: [Option<DataPid>; 0xFF] = [
    Some(DataPid { // 0x00
        pid: 0x00,
        required_size: 4,
        name: "PidSupport0120",
        desc: "PID support [01-20]",
        f: (|x| {
            Ok(get_obd_supported_pids(0x01, &x).iter()
                .map(|pid| {
                    let s_name = match &OBD_PIDS[*pid as usize] {
                        Some(service) => service.name.to_string(),
                        None => format!("OBD_UNK_PID_{:02X}", pid)
                    };
                    DataPresentation::from_enum("", s_name)
                })
                .collect()
            )
        })
    }),
    None, // 0x01
    None, // 0x02 
    None, // 0x03
    Some(DataPid {  // 0x04
        pid: 0x04,
        required_size: 1,
        name: "CalcEngineLoad",
        desc: "Calculated engine load",
        f: (|x| {
            Ok(vec![DataPresentation::from_number("Calculated engine load", (x[0] as f32)/2.55, Some("%".into()))])
        })
    }),
    Some(DataPid {  // 0x05
        pid: 0x05,
        required_size: 1,
        name: "CoolantTemp",
        desc: "Engine coolant temperature",
        f: (|x| {
            Ok(vec![DataPresentation::from_number("Engine coolant temperature", (x[0] as f32) - 40.0, Some("°C".into()))])
        })
    }),
    None, // 0x06
    None, // 0x07
    None, // 0x08
    None, // 0x09
    None, // 0x0A
    None, // 0x0B
    None, // 0x0C
    None, // 0x0D
    None, // 0x0E
    None, // 0x0F
    None, // 0x10
    None, // 0x11
    None, // 0x12
    None, // 0x13
    None, // 0x14
    None, // 0x15
    None, // 0x16
    None, // 0x17
    None, // 0x18
    None, // 0x19
    None, // 0x1A
    None, // 0x1B
    None, // 0x1C
    None, // 0x1D
    None, // 0x1E
    None, // 0x1F
    None, // 0x20
    None, // 0x21
    None, // 0x22
    None, // 0x23
    None, // 0x24
    None, // 0x25
    None, // 0x26
    None, // 0x27
    None, // 0x28
    None, // 0x29
    None, // 0x2A
    None, // 0x2B
    None, // 0x2C
    None, // 0x2D
    None, // 0x2E
    None, // 0x2F
    None, // 0x30
    None, // 0x31
    None, // 0x32
    None, // 0x33
    None, // 0x34
    None, // 0x35
    None, // 0x36
    None, // 0x37
    None, // 0x38
    None, // 0x39
    None, // 0x3A
    None, // 0x3B
    None, // 0x3C
    None, // 0x3D
    None, // 0x3E
    None, // 0x3F
    None, // 0x40
    None, // 0x41
    None, // 0x42
    None, // 0x43
    None, // 0x44
    None, // 0x45
    None, // 0x46
    None, // 0x47
    None, // 0x48
    None, // 0x49
    None, // 0x4A
    None, // 0x4B
    None, // 0x4C
    None, // 0x4D
    None, // 0x4E
    None, // 0x4F
    None, // 0x50
    None, // 0x51
    None, // 0x52
    None, // 0x53
    None, // 0x54
    None, // 0x55
    None, // 0x56
    None, // 0x57
    None, // 0x58
    None, // 0x59
    None, // 0x5A
    None, // 0x5B
    None, // 0x5C
    None, // 0x5D
    None, // 0x5E
    None, // 0x5F
    None, // 0x60
    None, // 0x61
    None, // 0x62
    None, // 0x63
    None, // 0x64
    None, // 0x65
    None, // 0x66
    None, // 0x67
    None, // 0x68
    None, // 0x69
    None, // 0x6A
    None, // 0x6B
    None, // 0x6C
    None, // 0x6D
    None, // 0x6E
    None, // 0x6F
    None, // 0x70
    None, // 0x71
    None, // 0x72
    None, // 0x73
    None, // 0x74
    None, // 0x75
    None, // 0x76
    None, // 0x77
    None, // 0x78
    None, // 0x79
    None, // 0x7A
    None, // 0x7B
    None, // 0x7C
    None, // 0x7D
    None, // 0x7E
    None, // 0x7F
    None, // 0x80
    None, // 0x81
    None, // 0x82
    None, // 0x83
    None, // 0x84
    None, // 0x85
    None, // 0x86
    None, // 0x87
    None, // 0x88
    None, // 0x89
    None, // 0x8A
    None, // 0x8B
    None, // 0x8C
    None, // 0x8D
    None, // 0x8E
    None, // 0x8F
    None, // 0x90
    None, // 0x91
    None, // 0x92
    None, // 0x93
    None, // 0x94
    None, // 0x95
    None, // 0x96
    None, // 0x97
    None, // 0x98
    None, // 0x99
    None, // 0x9A
    None, // 0x9B
    None, // 0x9C
    None, // 0x9D
    None, // 0x9E
    None, // 0x9F
    None, // 0xA0
    None, // 0xA1
    None, // 0xA2
    None, // 0xA3
    None, // 0xA4
    None, // 0xA5
    None, // 0xA6
    None, // 0xA7
    None, // 0xA8
    None, // 0xA9
    None, // 0xAA
    None, // 0xAB
    None, // 0xAC
    None, // 0xAD
    None, // 0xAE
    None, // 0xAF
    None, // 0xB0
    None, // 0xB1
    None, // 0xB2
    None, // 0xB3
    None, // 0xB4
    None, // 0xB5
    None, // 0xB6
    None, // 0xB7
    None, // 0xB8
    None, // 0xB9
    None, // 0xBA
    None, // 0xBB
    None, // 0xBC
    None, // 0xBD
    None, // 0xBE
    None, // 0xBF
    None, // 0xC0
    None, // 0xC1
    None, // 0xC2
    None, // 0xC3
    None, // 0xC4
    None, // 0xC5
    None, // 0xC6
    None, // 0xC7
    None, // 0xC8
    None, // 0xC9
    None, // 0xCA
    None, // 0xCB
    None, // 0xCC
    None, // 0xCD
    None, // 0xCE
    None, // 0xCF
    None, // 0xD0
    None, // 0xD1
    None, // 0xD2
    None, // 0xD3
    None, // 0xD4
    None, // 0xD5
    None, // 0xD6
    None, // 0xD7
    None, // 0xD8
    None, // 0xD9
    None, // 0xDA
    None, // 0xDB
    None, // 0xDC
    None, // 0xDD
    None, // 0xDE
    None, // 0xDF
    None, // 0xE0
    None, // 0xE1
    None, // 0xE2
    None, // 0xE3
    None, // 0xE4
    None, // 0xE5
    None, // 0xE6
    None, // 0xE7
    None, // 0xE8
    None, // 0xE9
    None, // 0xEA
    None, // 0xEB
    None, // 0xEC
    None, // 0xED
    None, // 0xEE
    None, // 0xEF
    None, // 0xF0
    None, // 0xF1
    None, // 0xF2
    None, // 0xF3
    None, // 0xF4
    None, // 0xF5
    None, // 0xF6
    None, // 0xF7
    None, // 0xF8
    None, // 0xF9
    None, // 0xFA
    None, // 0xFB
    None, // 0xFC
    None, // 0xFD
    None, // 0xFE
];