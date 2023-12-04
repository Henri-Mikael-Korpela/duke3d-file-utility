use std::{
    fs::File,
    io::{BufReader, Read, Seek},
};

/// File reader for .grp files, which are used by the Build engine.
///
/// See https://moddingwiki.shikadi.net/wiki/GRP_Format
pub struct GrpFileReader<'a> {
    pub file_count: u32,
    reader: BufReader<&'a File>,
}
impl<'a> GrpFileReader<'a> {
    const FORMAT_DESIGNER_NAME: &[u8; 12] = b"KenSilverman";
    const FILE_COUNT_BYTES: usize = 4;

    pub fn new(file: &'a File) -> Result<Self, String> {
        let mut reader = BufReader::new(file);

        // Ensure that the file is at least 12 bytes long
        // (the length of the magic constant) and that the
        // magic constant matches the one used by the Build engine.
        let mut format_designer_name_buf = [0u8; 12];
        reader
            .read_exact(&mut format_designer_name_buf)
            .map_err(|_| "Failed to read magic constant from .grp file.")?;

        if format_designer_name_buf != *Self::FORMAT_DESIGNER_NAME {
            return Err(format!(
                "Magic constant \"{}\" does not match the magic \"{}\" read from the .grp file.",
                String::from_utf8_lossy(&format_designer_name_buf),
                String::from_utf8_lossy(Self::FORMAT_DESIGNER_NAME)
            ));
        }

        // Read the file count. The file count is stored
        // as a little-endian unsigned 32-bit integer.
        let file_count = {
            let mut file_count_buf = [0u8; Self::FILE_COUNT_BYTES];
            reader.read_exact(&mut file_count_buf).map_err(|_| {
                "Failed to read file count from .grp file. There are not enough bytes in the file for reading."
            })?;
            u32::from_le_bytes(file_count_buf)
        };

        Ok(Self { file_count, reader })
    }
    /// A shorthand for getting the file entries and finding a specific file entry among them.
    pub fn find_file_entry(&mut self, file_name: &str) -> Result<Option<GrpFileEntry>, String> {
        let file_entries = self.get_file_entries()?;
        Ok(file_entries.into_iter().find(|f| f.name() == file_name))
    }
    pub fn get_file_entries(&mut self) -> Result<Vec<GrpFileEntry>, String> {
        // Ensure the file reader is set after the format designer name and the file count.
        self.reader
            .seek(std::io::SeekFrom::Start(
                (Self::FORMAT_DESIGNER_NAME.len() + Self::FILE_COUNT_BYTES) as u64,
            ))
            .map_err(|_| {
                "Failed to set the file reader after the format designer name and the file count."
            })?;

        let mut current_offset = (Self::FORMAT_DESIGNER_NAME.len()
            + Self::FILE_COUNT_BYTES
            + self.file_count as usize * 16) as u64;
        let mut files = Vec::with_capacity(self.file_count as usize);

        // Read the file entries based on the file count.
        for _ in 0..self.file_count {
            // Read the file name. The max length of the file name is 12 bytes.
            // If the file name is shorter than 12 bytes, the remaining bytes
            // are filled with null bytes.
            let mut file_name_buf = [0u8; 12];
            self.reader
                .read_exact(&mut file_name_buf)
                .map_err(|_| "Failed to read file name from .grp file.")?;

            // Read the file size. The file size is stored
            // as a little-endian unsigned 32-bit integer.
            let file_size = {
                let mut size_buf = [0u8; 4];
                self.reader
                    .read_exact(&mut size_buf)
                    .map_err(|_| "Failed to read file size from .grp file.")?;
                u32::from_le_bytes(size_buf)
            };

            files.push(GrpFileEntry {
                name: file_name_buf,
                offset: current_offset,
                size: file_size,
            });

            current_offset += file_size as u64;
        }

        Ok(files)
    }
    pub fn read_file(&mut self, entry: &GrpFileEntry) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; entry.size as usize];
        self.reader
            .seek(std::io::SeekFrom::Start(entry.offset))
            .map_err(|_| "Failed to seek to file offset.")?;
        self.reader
            .read_exact(&mut buf)
            .map_err(|_| "Failed to read file from .grp file.")?;
        Ok(buf)
    }
}

#[derive(Debug)]
pub struct GrpFileEntry {
    name: [u8; 12],
    offset: u64,
    size: u32,
}
impl GrpFileEntry {
    pub fn name(&self) -> String {
        let mut name = String::with_capacity(12);
        for byte in &self.name {
            if *byte == 0 {
                break;
            }
            name.push(*byte as char);
        }
        name
    }
}
