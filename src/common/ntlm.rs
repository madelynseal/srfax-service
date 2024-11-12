// https://innovation.ch/personal/ronald/ntlm.html

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Fail)]
#[fail(display = "ntlm header is invalid")]
pub struct HeaderInvalid;

#[derive(Debug, Fail)]
#[fail(display = "tried to parse message type {}, got type {}", expected, got)]
pub struct WrongType {
    expected: u8,
    got: u8,
}

pub const TYPE_ONE_MESSAGE: u8 = 0x01;
pub const TYPE_TWO_MESSAGE: u8 = 0x02;
pub const TYPE_THREE_MESSAGE: u8 = 0x03;

const TYPE: usize = 8;

const T1_FLAGS: usize = 12;
const T1_DOM_LEN: usize = 16;
const T1_DOM_OFFSET: usize = 20;
const T1_HOST_LEN: usize = 24;
const T1_HOST_OFFSET: usize = 28;

const T2_FLAGS: usize = 20;
const T2_NONCE: usize = 24;

const T3_LM_RESP_LEN: usize = 12;
const T3_LM_RESP_OFFSET: usize = 16;
const T3_NT_RESP_LEN: usize = 20;
const T3_NT_RESP_OFFSET: usize = 24;
const T3_DOMAIN_LEN: usize = 28;
const T3_DOMAIN_OFFSET: usize = 32;
const T3_USER_LEN: usize = 36;
const T3_USER_OFFSET: usize = 40;
const T3_HOST_LEN: usize = 44;
const T3_HOST_OFFSET: usize = 48;
const T3_MESSAGE_LEN: usize = 56;
const T3_FLAGS: usize = 60;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Protocol {
    NTLM_SSP,
}

#[derive(Debug)]
pub struct Type1Message {
    pub protocol: Protocol,
    pub mtype: u8,      // 0x01
    pub flags: u16,     // usually 0xb203
    pub host: String,   // ASCII
    pub domain: String, // ASCII
}
impl Type1Message {
    pub fn from_raw(data: &[u8]) -> Result<Self> {
        if &data[..8] != "NTLMSSP\0".as_bytes() {
            return Err(HeaderInvalid.into());
        }
        if data[TYPE] != TYPE_ONE_MESSAGE {
            return Err(WrongType {
                expected: TYPE_ONE_MESSAGE,
                got: data[TYPE],
            }
            .into());
        }

        let host_len = get_short(data, T1_HOST_LEN) as usize;
        let host_offset = data[T1_HOST_OFFSET] as usize;

        let dom_len = get_short(data, T1_DOM_LEN) as usize;
        let dom_offset = data[T1_DOM_OFFSET] as usize;

        Ok(Self {
            protocol: Protocol::NTLM_SSP,
            mtype: data[TYPE],
            flags: get_short(data, T1_FLAGS),
            domain: get_utf8_string(data, dom_offset, dom_len),
            host: get_utf8_string(data, host_offset, host_len),
        })
    }
}

#[derive(Debug)]
pub struct Type2Message {
    pub protocol: Protocol,
    pub mtype: u8,  // 0x02
    pub flags: u16, // usually 0x8201
    pub nonce: [u8; 8],
}
impl Type2Message {
    pub fn from_raw(data: &[u8]) -> Result<Self> {
        if &data[..8] != "NTLMSSP\0".as_bytes() {
            return Err(HeaderInvalid.into());
        }
        if data[TYPE] != TYPE_TWO_MESSAGE {
            return Err(WrongType {
                expected: TYPE_TWO_MESSAGE,
                got: data[TYPE],
            }
            .into());
        }

        Ok(Self {
            protocol: Protocol::NTLM_SSP,
            mtype: data[TYPE],
            flags: get_short(data, T2_FLAGS),
            nonce: get_array8(data, T2_NONCE),
        })
    }
}

#[derive(Debug)]
pub struct Type3Message {
    pub protocol: Protocol,
    pub mtype: u8,        // 0x03
    pub flags: u16,       // usually 0x8201
    pub domain: String,   // utf-16 le
    pub user: String,     // utf-16 le
    pub host: String,     // utf-16 le
    pub lm_resp: Vec<u8>, // LanManager Response
    pub nt_resp: Vec<u8>, // NT Response
}

impl Type3Message {
    pub fn from_header_base64(header: &str) -> Result<Self> {
        if !header.starts_with("NTLM ") {
            return Err(HeaderInvalid.into());
        }
        let header = &header[5..];

        let a: Vec<u8> = base64::decode(header)?;

        Ok(Type3Message::from_raw(&a)?)
    }

    pub fn from_raw(a: &[u8]) -> Result<Self> {
        if &a[..8] != "NTLMSSP\0".as_bytes() {
            return Err(HeaderInvalid.into());
        }
        if a[TYPE] != TYPE_THREE_MESSAGE {
            return Err(WrongType {
                expected: TYPE_THREE_MESSAGE,
                got: a[TYPE],
            }
            .into());
        }

        let lm_resp_len = get_short(&a, T3_LM_RESP_LEN) as usize;
        let lm_resp_offset = a[T3_LM_RESP_OFFSET] as usize;

        let nt_resp_len = get_short(&a, T3_NT_RESP_LEN) as usize;
        let nt_resp_offset = a[T3_NT_RESP_OFFSET] as usize;

        let domain_len = get_short(&a, T3_DOMAIN_LEN) as usize;
        let domain_offset = a[T3_DOMAIN_OFFSET] as usize;

        let user_len = get_short(&a, T3_USER_LEN) as usize;
        let user_offset = a[T3_USER_OFFSET] as usize;

        let host_len = get_short(&a, T3_HOST_LEN) as usize;
        let host_offset = a[T3_HOST_OFFSET] as usize;

        Ok(Self {
            protocol: Protocol::NTLM_SSP,
            mtype: a[TYPE],
            flags: get_short(&a, T3_FLAGS),
            domain: get_utf16_string(&a, domain_offset, domain_len),
            user: get_utf16_string(&a, user_offset, user_len),
            host: get_utf16_string(&a, host_offset, host_len),
            lm_resp: a[lm_resp_offset..(lm_resp_offset + lm_resp_len)].to_vec(),
            nt_resp: a[nt_resp_offset..(nt_resp_offset + nt_resp_len)].to_vec(),
        })
    }
}

fn get_short(v: &[u8], offset: usize) -> u16 {
    let b1 = u16::from(u8::to_le(v[offset]));
    let b2 = u16::from(u8::to_le(v[offset + 1]));

    b1 | (b2 << 8)
}

fn get_utf16_string(v: &[u8], offset: usize, len: usize) -> String {
    let mut buf: Vec<u16> = vec![];

    for i in 0..(len / 2) {
        buf.push(get_short(v, offset + (i * 2)));
    }

    String::from_utf16_lossy(&buf)
}
fn get_utf8_string(v: &[u8], offset: usize, len: usize) -> String {
    String::from_utf8_lossy(&v[offset..(offset + len)]).to_string()
}

fn get_array8(v: &[u8], offset: usize) -> [u8; 8] {
    [
        v[offset],
        v[offset + 1],
        v[offset + 2],
        v[offset + 3],
        v[offset + 4],
        v[offset + 5],
        v[offset + 6],
        v[offset + 7],
    ]
}

pub fn get_type_base64(bdata: &[u8]) -> Result<u8> {
    let bdata = if bdata.starts_with(b"NTLM ") {
        &bdata[5..]
    } else {
        bdata
    };

    let data = base64::decode(&bdata)?;

    Ok(data[TYPE])
}
pub fn get_type(data: &[u8]) -> u8 {
    data[TYPE]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_short1() {
        let vec: Vec<u8> = vec![0xA, 0xA];

        assert_eq!(get_short(&vec, 0), 2570);
    }

}
