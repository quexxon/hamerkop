pub mod ips {
    use std::error::Error;
    use std::fmt::{self, Display, Formatter};
    use std::fs;
    use std::path::Path;

    #[derive(Debug)]
    enum Data {
        Standard(Vec<u8>),
        RLE { size: u16, value: u8 },
    }

    #[derive(Debug)]
    struct Record {
        offset: u32,
        data: Data,
    }

    impl Record {
        fn new(file: &[u8], start_index: usize) -> (Self, usize) {
            let mut index = start_index;

            let offset = Self::get_offset(&file[index..]);
            index += 3;

            let data_size = Self::get_data_size(&file[index..]);
            index += 2;

            let data = if data_size == 0 {
                Data::RLE {
                    size: Self::get_data_size(&file[index..]),
                    value: file[index + 2],
                }
            } else {
                Data::Standard(file[index..(index + data_size as usize)].to_vec())
            };

            if data_size == 0 {
                index += 3;
            } else {
                index += data_size as usize;
            }

            (Self { offset, data }, index)
        }

        fn apply_patch(&self, buffer: &mut Vec<u8>) {
            let mut index = self.offset as usize;

            match &self.data {
                Data::Standard(data) => {
                    for value in data.iter() {
                        if index == buffer.len() {
                            buffer.push(*value);
                        } else {
                            buffer[index] = *value;
                        }
                        index += 1;
                    }
                }

                Data::RLE { size, value } => {
                    for _ in 0..*size {
                        if index == buffer.len() {
                            buffer.push(*value);
                        } else {
                            buffer[index] = *value;
                        }
                        index += 1;
                    }
                }
            }
        }

        fn get_offset(bytes: &[u8]) -> u32 {
            (u32::from(bytes[0]) << 16) + (u32::from(bytes[1]) << 8) + u32::from(bytes[2])
        }

        fn get_data_size(bytes: &[u8]) -> u16 {
            (u16::from(bytes[0]) << 8) + u16::from(bytes[1])
        }
    }

    #[derive(Debug)]
    pub struct InvalidFormatError(InvalidFormatCause);

    impl Display for InvalidFormatError {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            write!(f, "IPS file has invalid format")
        }
    }

    impl Error for InvalidFormatError {
        fn description(&self) -> &str {
            "Given patch file doesn't conform to IPS format"
        }

        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.0)
        }
    }

    #[derive(Debug)]
    pub enum InvalidFormatCause {
        MissingHeader,
        MissingEOF,
    }

    impl Display for InvalidFormatCause {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self {
                InvalidFormatCause::MissingHeader => write!(f, "IPS file has no header"),
                InvalidFormatCause::MissingEOF => write!(f, "IPS file has no EOF marker"),
            }
        }
    }

    impl Error for InvalidFormatCause {
        fn description(&self) -> &str {
            match self {
                InvalidFormatCause::MissingHeader => "IPS file has no header",
                InvalidFormatCause::MissingEOF => "IPS file has no EOF marker",
            }
        }
    }

    pub struct IPS {
        buffer: Vec<u8>,
        records: Vec<Record>,
    }

    impl IPS {
        pub fn parse<P: AsRef<Path>>(
            input: P,
        ) -> Result<Self, Box<dyn std::error::Error + 'static>> {
            let mut ips = Self {
                buffer: fs::read(input)?,
                records: Vec::new(),
            };

            if &ips.buffer[0..5] != b"PATCH" {
                return Err(InvalidFormatError(InvalidFormatCause::MissingHeader).into());
            }

            let eof_start = ips.buffer.len() - 3;
            if &ips.buffer[eof_start..ips.buffer.len()] != b"EOF" {
                return Err(InvalidFormatError(InvalidFormatCause::MissingEOF).into());
            }

            let mut index = 5;
            while index < eof_start {
                let (record, next_index) = Record::new(&ips.buffer, index);
                ips.records.push(record);
                index = next_index;
            }

            Ok(ips)
        }

        pub fn apply<P: AsRef<Path>>(
            &self,
            input: P,
            output: P,
        ) -> Result<(), Box<dyn std::error::Error + 'static>> {
            let mut buffer = fs::read(input)?;

            for record in self.records.iter() {
                record.apply_patch(&mut buffer);
            }

            fs::write(output, &buffer)?;
            Ok(())
        }
    }
}
