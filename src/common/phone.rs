use serde::{de::Error, de::Visitor, Deserialize, Deserializer};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct PhoneNumber {
    num: u64,
}

#[allow(dead_code)]
impl PhoneNumber {
    pub fn new(num: u64) -> PhoneNumber {
        PhoneNumber { num }
    }

    pub fn first_three(&self) -> u16 {
        (self.num / 10_000_000_000) as u16
    }
    pub fn second_three(&self) -> u16 {
        ((self.num / 10_000_000) - (self.num / 10_000_000_000) * 1000) as u16
    }
    pub fn second_last_three(&self) -> u16 {
        ((self.num / 10_000) - ((self.num / 10_000_000) * 1000)) as u16
    }
    pub fn last_four(&self) -> u16 {
        (self.num - ((self.num / 10_000) * 10_000)) as u16
    }
    pub fn get_pair(&self, pnum: u8) -> u8 {
        let n = u32::from(pnum * 2 + 2);
        let upper = 10u64.pow(n);
        let lower = 10u64.pow(n - 2);

        ((self.num / lower) - ((self.num / upper) * 100)) as u8
    }

    pub fn format_na(&self) -> String {
        let country_code = self.first_three();
        if country_code > 0 {
            format!(
                "{}-{:03}-{:03}-{:04}",
                country_code,
                self.second_three(),
                self.second_last_three(),
                self.last_four()
            )
        } else {
            format!(
                "{:03}-{:03}-{:04}",
                self.second_three(),
                self.second_last_three(),
                self.last_four()
            )
        }
    }
    pub fn format_fr(&self) -> String {
        format!(
            "{:02}-{:02}-{:02}-{:02}-{:02}-{:02}-{:02}",
            self.get_pair(6),
            self.get_pair(5),
            self.get_pair(4),
            self.get_pair(3),
            self.get_pair(2),
            self.get_pair(1),
            self.get_pair(0)
        )
    }
}

#[derive(Debug, Fail, PartialEq, Eq)]
pub enum PhoneNumberError {
    #[fail(display = "PhoneNumber(Parse({}))", _0)]
    Parse(#[cause] std::num::ParseIntError),

    #[fail(display = "PhoneNumber(invalid character found)")]
    InvalidChar,

    #[fail(display = "PhoneNumber(number too big)")]
    Range,
}

impl std::str::FromStr for PhoneNumber {
    type Err = PhoneNumberError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = String::new();
        for c in s.chars() {
            match c {
                '-' | '+' | ' ' => {}
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => buf.push(c),
                _ => return Err(PhoneNumberError::InvalidChar),
            }
        }

        if buf.len() > 13 {
            return Err(PhoneNumberError::Range);
        }

        Ok(PhoneNumber {
            num: buf.parse().map_err(PhoneNumberError::Parse)?,
        })
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.format_na())?;
        Ok(())
    }
}

impl<'de> Deserialize<'de> for PhoneNumber {
    fn deserialize<D>(deserializer: D) -> Result<PhoneNumber, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            struct PhoneNumberVisitor;
            impl<'de> Visitor<'de> for PhoneNumberVisitor {
                type Value = PhoneNumber;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("phone number")
                }
                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    s.parse().map_err(Error::custom)
                }
            }

            deserializer.deserialize_str(PhoneNumberVisitor)
        } else {
            <PhoneNumber>::deserialize(deserializer).map(<PhoneNumber>::from)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn phone_get_sections() {
        let p = PhoneNumber { num: 11234567890 };

        assert_eq!(p.first_three(), 1);
        assert_eq!(p.second_three(), 123);
        assert_eq!(p.second_last_three(), 456);
        assert_eq!(p.last_four(), 7890);
    }
    #[test]
    fn phone_get_pairs() {
        let p = PhoneNumber { num: 11234567890 };

        assert_eq!(p.get_pair(0), 90);
        assert_eq!(p.get_pair(1), 78);
        assert_eq!(p.get_pair(2), 56);
        assert_eq!(p.get_pair(3), 34);
        assert_eq!(p.get_pair(4), 12);
        assert_eq!(p.get_pair(5), 01);
        assert_eq!(p.get_pair(6), 00);
    }

    #[test]
    fn phone_number_short() -> Result<(), PhoneNumberError> {
        let s = "123-456-7890";

        let phone: PhoneNumber = s.parse()?;

        assert_eq!(phone, PhoneNumber { num: 1234567890 });
        assert_eq!(&phone.to_string(), "123-456-7890");

        Ok(())
    }
    #[test]
    fn phone_number_long1() -> Result<(), PhoneNumberError> {
        let s = "1-123-456-7890";

        let phone: PhoneNumber = s.parse()?;

        assert_eq!(phone, PhoneNumber { num: 11234567890 });
        assert_eq!(&phone.to_string(), "1-123-456-7890");

        Ok(())
    }
    #[test]
    fn phone_number_long2() -> Result<(), PhoneNumberError> {
        let s = "1123-456-7890";

        let phone: PhoneNumber = s.parse()?;

        assert_eq!(phone, PhoneNumber { num: 11234567890 });
        assert_eq!(&phone.to_string(), "1-123-456-7890");

        Ok(())
    }
    #[test]
    fn phone_number_format_fr() {
        let phone = PhoneNumber { num: 11234567890 };

        assert_eq!(phone.format_fr(), "00-01-12-34-56-78-90");
    }
    #[test]
    fn phone_number_invalid_char() {
        let s = "1-123-456-abcd";

        let phone: Result<PhoneNumber, PhoneNumberError> = s.parse();

        assert_eq!(phone, Err(PhoneNumberError::InvalidChar));
    }
}
