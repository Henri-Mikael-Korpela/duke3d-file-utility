use std::{
    fs::File,
    io::{BufReader, Read, Seek},
};

pub struct ArtFileReader<'a> {
    reader: BufReader<&'a File>,
}
impl<'a> ArtFileReader<'a> {
    pub fn new(file: &'a File) -> Result<Self, String> {
        let mut reader = BufReader::new(file);

        // Ensure the header contains valid version number.
        // Read the version number as a little-endian 32-bit unsigned integer.
        const SUPPORTED_VERSION_NUMBER: u32 = 1;

        let mut version_number = [0u8; 4];
        reader
            .read_exact(&mut version_number)
            .map_err(|_| "Failed to read version number from .art file.")?;
        let version_number = u32::from_le_bytes(version_number);

        if version_number != SUPPORTED_VERSION_NUMBER {
            return Err(format!(
                "Unsupported version number {} (should be {})",
                version_number, SUPPORTED_VERSION_NUMBER
            ));
        }

        // The header contains the number of tiles in the file,
        // but there is no need to read it.

        Ok(Self { reader })
    }
    pub fn read_tiles(&mut self) -> Result<Vec<ArtTile>, String> {
        // Ensure the file reader is set after the version number and the number of tiles.
        self.reader
            .seek(std::io::SeekFrom::Start(4 + 4))
            .map_err(|_| {
                "Failed to set the file reader after the version number and the number of tiles."
            })?;

        // Read the number of the first tile (localtilestart).
        // Read the number of the first tile as a little-endian 32-bit unsigned integer.
        let mut first_tile_number = [0u8; 4];
        self.reader
            .read_exact(&mut first_tile_number)
            .map_err(|_| "Failed to read first tile number from .art file.")?;
        let first_tile_number = u32::from_le_bytes(first_tile_number);

        // Read the number of the last tile (localtileend).
        // Read the number of the last tile as a little-endian 32-bit unsigned integer.
        let mut last_tile_number = [0u8; 4];
        self.reader
            .read_exact(&mut last_tile_number)
            .map_err(|_| "Failed to read last tile number from .art file.")?;
        let last_tile_number = u32::from_le_bytes(last_tile_number);

        let tile_count = last_tile_number - first_tile_number + 1;

        // Read x-dimensions of all of the tiles in the file.
        // Each x-dimension is stored as a little-endian 16-bit signed integer.
        let mut tile_widths = Vec::with_capacity(tile_count as usize);
        for _ in 0..tile_count {
            let mut tile_width = [0u8; 2];
            self.reader
                .read_exact(&mut tile_width)
                .map_err(|_| "Failed to read tile width from .art file.")?;
            let tile_width = i16::from_le_bytes(tile_width);
            tile_widths.push(tile_width);
        }

        // Read y-dimensions of all of the tiles in the file.
        // Each y-dimension is stored as a little-endian 16-bit signed integer.
        let mut tile_heights = Vec::with_capacity(tile_count as usize);
        for _ in 0..tile_count {
            let mut tile_height = [0u8; 2];
            self.reader
                .read_exact(&mut tile_height)
                .map_err(|_| "Failed to read tile height from .art file.")?;
            let tile_height = i16::from_le_bytes(tile_height);
            tile_heights.push(tile_height);
        }

        // "Merge" the tile widths and heights together into a vector or tiles.
        let tiles = tile_widths
            .iter()
            .zip(tile_heights.iter())
            .enumerate()
            .map(|(i, (w, h))| ArtTile {
                height: *h,
                number: first_tile_number + i as u32,
                width: *w,
            })
            .collect::<Vec<_>>();

        Ok(tiles)
    }
}

#[derive(Debug)]
pub struct ArtTile {
    height: i16,
    number: u32,
    width: i16,
}

#[test]
fn should_read_art() {
    let curr_dir = std::env::current_dir().unwrap();
    let file_path = curr_dir.join("../tmp/TILES000.ART");
    let file = File::open(file_path).unwrap();
    let mut art_reader = ArtFileReader::new(&file).unwrap();
    let tiles = art_reader.read_tiles().unwrap();
    println!(
        "tiles: {:#?}",
        tiles[0..15]
            .iter()
            .map(|t| format!("{:0>4}: {}x{}", t.number, t.width, t.height))
            .collect::<Vec<_>>()
    );
}
