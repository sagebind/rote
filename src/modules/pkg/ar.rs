use std::io::{Read, Result, Write};


/// An entry in an AR archive.
pub struct Entry<T: Read> {
    name: [u8; 16],
    timestamp: [u8; 12],
    owner_id: [u8; 6],
    group_id: [u8; 6],
    mode: [u8; 8],
    size: [u8; 10],
    stream: T,
}

impl<T: Read> Entry<T> {
    /// Creates a new entry with a given readable stream for file contents.
    pub fn new(stream: T) -> Entry<T> {
        Entry {
            name: [32; 16],
            timestamp: [32; 12],
            owner_id: [32; 6],
            group_id: [32; 6],
            mode: [32; 8],
            size: [32; 10],
            stream: stream,
        }.timestamp(0).owner_id(0).group_id(0).mode(0o644).size(0)
    }

    /// Sets the file name of the entry.
    pub fn name(mut self, name: &str) -> Self {
        let name = name.to_string() + "/";

        self.name = [32; 16];
        for (i, byte) in name.as_bytes().iter().enumerate() {
            self.name[i] = *byte;
        }

        self
    }

    /// Sets the file modification time stamp.
    pub fn timestamp(mut self, timestamp: u32) -> Self {
        self.timestamp = [32; 12];
        for (i, byte) in timestamp.to_string().as_bytes().iter().enumerate() {
            self.timestamp[i] = *byte;
        }

        self
    }

    /// Sets the owner user ID.
    pub fn owner_id(mut self, owner_id: u32) -> Self {
        self.owner_id = [32; 6];
        for (i, byte) in owner_id.to_string().as_bytes().iter().enumerate() {
            self.owner_id[i] = *byte;
        }

        self
    }

    /// Sets the group ID.
    pub fn group_id(mut self, group_id: u32) -> Self {
        self.group_id = [32; 6];
        for (i, byte) in group_id.to_string().as_bytes().iter().enumerate() {
            self.group_id[i] = *byte;
        }

        self
    }

    /// Sets the file mode.
    pub fn mode(mut self, mode: u32) -> Self {
        let mode = format!("{:o}", mode);

        self.mode = [32; 8];
        for (i, byte) in mode.as_bytes().iter().enumerate() {
            self.mode[i] = *byte;
        }

        self
    }

    /// Sets the file size in bytes.
    pub fn size(mut self, size: usize) -> Self {
        self.size = [32; 10];
        for (i, byte) in size.to_string().as_bytes().iter().enumerate() {
            self.size[i] = *byte;
        }

        self
    }
}


/// An AR archive file.
///
/// Currently only designed for writing AR archives, and not reading.
pub struct Ar<T: Write> {
    stream: T,
}

impl<T: Write> Ar<T> {
    /// Creates a new AR archive for writing.
    ///
    /// The archive contents will be written to the given stream as it becomes available.
    pub fn new(mut stream: T) -> Result<Ar<T>> {
        try!(stream.write_all(b"!<arch>\n"));

        Ok(Ar {
            stream: stream,
        })
    }

    /// Appends a file entry to the archve.
    pub fn append<U: Read>(&mut self, mut entry: Entry<U>) -> Result<()> {
        try!(self.stream.write_all(&entry.name));
        try!(self.stream.write_all(&entry.timestamp));
        try!(self.stream.write_all(&entry.owner_id));
        try!(self.stream.write_all(&entry.group_id));
        try!(self.stream.write_all(&entry.mode));

        let mut buffer = Vec::new();
        try!(entry.stream.read_to_end(&mut buffer));
        entry = entry.size(buffer.len());

        try!(self.stream.write_all(&entry.size));
        try!(self.stream.write_all(b"\x60\x0a"));
        try!(self.stream.write_all(&buffer));

        // 2-byte alignment padding
        if buffer.len() % 2 != 0 {
            try!(self.stream.write_all(b"\n"));
        }

        Ok(())
    }
}
