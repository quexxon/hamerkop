#[derive(Debug)]
enum Data<'a> {
    Standard(&'a [u8]),
    RLE { size: u16, value: u8 },
}

#[derive(Debug)]
pub struct IpsRecord<'a> {
    offset: u32,
    data: Data<'a>,
}

impl<'a> IpsRecord<'a> {
    pub fn new(file: &'a [u8], start_index: usize) -> (Self, usize) {
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
            Data::Standard(&file[index..(index + data_size as usize)])
        };

        if data_size == 0 {
            index += 3;
        } else {
            index += data_size as usize;
        }

        (Self { offset, data }, index)
    }

    pub fn apply_patch(&self, buffer: &mut Vec<u8>) {
        let mut index = self.offset as usize;

        match self.data {
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
                for _ in 0..size {
                    if index == buffer.len() {
                        buffer.push(value);
                    } else {
                        buffer[index] = value;
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
