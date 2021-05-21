use byteorder::{NativeEndian, ReadBytesExt};
use integer_encoding::VarIntReader;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::Path,
    boxed::Box,
};
use zstd::stream::read::Decoder;

pub const SEG_SIZE: usize = 16 * 8;
pub const CHUNK_HEIGHT: usize = 255;

enum SegmentState {
    Start, // fresh file
    Middle, // done with the palette
    Done // done with everything
}

pub struct SegmentReader {
    f: Option<BufReader<Box<dyn Read>>>,
    state: SegmentState,
}

pub struct PaletteIterator<'a> {
    f: Option<BufReader<Box<dyn Read>>>,
    file_owner: &'a mut SegmentReader,
}

pub struct BlockIterator<'a> {
    f: Option<BufReader<Box<dyn Read>>>,
    file_owner: &'a mut SegmentReader,
    off: usize,
}

impl SegmentReader {
    pub fn new(name: &str) -> SegmentReader {
        let mut f = File::open(&Path::new(
            format!("./public/segments/{}", name).as_str())).unwrap();

        let mut magic_buf: [u8; 4] = [0; 4];
        let magic: [u8; 4] = [ 0x28, 0xb5, 0x2f, 0xfd ];
        f.read_exact(&mut magic_buf).unwrap();
        let is_compressed = magic_buf.iter()
            .zip(magic.iter())
            .fold(true, |a, (x,y)| a && x == y);
        f.seek(SeekFrom::Start(0)).unwrap();

        let reader_box: Box<dyn Read> = match is_compressed {
            false => Box::new(f),
            true => Box::new(Decoder::new(f).unwrap())
        };

        SegmentReader{
            f: Some(BufReader::new(reader_box)),
            state: SegmentState::Start
        }
    }

    pub fn iter_palette(&mut self) -> PaletteIterator {
        match self.state {
            SegmentState::Start => (),
            _ => panic!("Wrong SegmentReader state on iter_palette")
        }
        assert!(self.f.is_some());
        PaletteIterator{
            f: Some(self.f.take().unwrap()),
            file_owner: self
        }
    }

    pub fn iter_blocks(&mut self) -> BlockIterator {
        match self.state {
            SegmentState::Middle => (),
            _ => panic!("Wrong SegmentReader state on iter_palette")
        }
        assert!(self.f.is_some());
        BlockIterator{
            f: Some(self.f.take().unwrap()),
            file_owner: self,
            off: 0
        }
    }
}

impl Iterator for PaletteIterator<'_> {
    type Item = (String, u16);

    fn next(&mut self) -> Option<Self::Item> {
        let f = self.f.as_mut().unwrap();

        let mut block_name_raw = vec![];
        f.read_until(0x00, &mut block_name_raw).unwrap();
        block_name_raw.pop();
        let block_id = f.read_u16::<NativeEndian>().unwrap();

        if block_name_raw.len() == 0 {
            self.file_owner.f = self.f.take();
            self.file_owner.state = SegmentState::Middle;
            return None;
        }
        let block_name = String::from_utf8(block_name_raw).unwrap();

        return Some((block_name, block_id));
    }
}

impl Iterator for BlockIterator<'_> {
    type Item = (i32, i32, i32, u16);

    fn next(&mut self) -> Option<Self::Item> {
        let f = self.f.as_mut().unwrap();

        match f.read_varint::<usize>() {
            Err(_e) => {
                self.file_owner.f = self.f.take();
                self.file_owner.state = SegmentState::Done;
                return None;
            },
            Ok(dist) => {
                self.off += dist;
                let x = f.read_u16::<NativeEndian>().unwrap();

                // first bit is whether the distance to the last block is filled
                // with air or removed, but we don't expose that here
                let block_id = x & 0x7fff;
                let (x, y, z) = zseg_index_to_xyz(self.off);

                self.off += 1;

                return Some((x, y, z, block_id));
            }
        }
    }
}

const fn zseg_index_to_xyz(n: usize) -> (i32, i32, i32) {
    let (x, z, y) = (
        n % SEG_SIZE,
       (n / SEG_SIZE) % SEG_SIZE,
        n / SEG_SIZE / SEG_SIZE
    );

    (x as i32, y as i32, z as i32)
}
