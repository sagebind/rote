use runtime::error::Error;
use flate2::Compression;
use flate2::write::GzEncoder;
use pkg::ar;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use tar;
use time;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;


/// An enum of allowed package target architectures.
pub enum Arch {
    X86,
    X64,
    All,
}

impl ToString for Arch {
    fn to_string(&self) -> String {
        match *self {
            Arch::X86 => "i386",
            Arch::X64 => "amd64",
            Arch::All => "all",
        }
        .to_string()
    }
}


/// An enum of allowed package priorities.
pub enum Priority {
    Essential,
    Extra,
    Important,
    Optional,
    Required,
    Standard,
}

impl ToString for Priority {
    fn to_string(&self) -> String {
        match *self {
            Priority::Essential => "essential",
            Priority::Extra => "extra",
            Priority::Important => "important",
            Priority::Optional => "optional",
            Priority::Required => "required",
            Priority::Standard => "standard",
        }
        .to_string()
    }
}


/// A representation of a Debian binary package file.
pub struct Package {
    pub name: String,
    pub files: Vec<(PathBuf, PathBuf)>,
    pub priority: Priority,
    pub section: String,
    pub depends: Vec<(String, String)>,
    pub maintainer: String,
    pub arch: Arch,
    pub version: String,
    pub homepage: Option<String>,
    pub short_desc: String,
    pub long_desc: Option<String>,
}

impl Package {
    /// Writes a Debian package to a stream.
    pub fn write_to<T: Write>(&self, stream: &mut T) {
        let mut ar = ar::Ar::new(stream).unwrap();

        ar.append(ar::Entry::new("2.0\n".as_bytes()).name("debian-binary"))
          .unwrap();

        let control_archive = self.create_control_archive();
        ar.append(ar::Entry::new(&control_archive as &[u8]).name("control.tar.gz"))
          .unwrap();

        let data_archive = self.create_data_archive();
        ar.append(ar::Entry::new(&data_archive as &[u8]).name("data.tar.gz"))
          .unwrap();
    }

    /// Creates the control archive file.
    fn create_control_archive(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        {
            // Create a tar wrapped in a gzip encoder.
            let encoder = GzEncoder::new(&mut buffer, Compression::Default);
            let archive = tar::Archive::new(encoder);

            let control_file = self.create_control_file();

            let mut header = tar::Header::new();
            header.set_path("control").unwrap();
            header.set_size(control_file.len() as u64);
            header.set_mode(0o644);
            header.set_mtime(time::now().to_timespec().sec as u64);
            header.set_cksum();
            archive.append(&header, &mut control_file.as_bytes()).unwrap();

            archive.finish().unwrap();
            archive.into_inner().finish().unwrap();
        }

        buffer
    }

    /// Creates the data archive file.
    fn create_data_archive(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        {
            // Create a tar wrapped in a gzip encoder.
            let encoder = GzEncoder::new(&mut buffer, Compression::Default);
            let archive = tar::Archive::new(encoder);

            for paths in &self.files {
                let mut file = File::open(&paths.0).unwrap();

                let mut header = tar::Header::new();
                header.set_metadata(&paths.0.metadata().unwrap());
                header.set_path(&paths.1).unwrap();
                header.set_mode(0o755);
                header.set_uid(0);
                header.set_gid(0);
                header.set_cksum();
                archive.append(&header, &mut file).unwrap();
            }

            archive.finish().unwrap();
            archive.into_inner().finish().unwrap();
        }

        buffer
    }

    /// Generates a control file from the package metadata.
    fn create_control_file(&self) -> String {
        let mut string = String::new();

        string.push_str(&format!("Package: {}\n", self.name));
        string.push_str(&format!("Priority: {}\n", self.priority.to_string()));
        string.push_str(&format!("Section: {}\n", self.section));
        string.push_str(&format!("Installed-Size: {}\n", self.get_size()));

        if !self.depends.is_empty() {
            string.push_str("Depends: ");
            let mut start = true;

            for &(ref package, ref version) in &self.depends {
                if !start {
                    string.push_str(", ");
                } else {
                    start = false;
                }

                string.push_str(&format!("{} ({})", package, version));
            }

            string.push_str("\n");
        }

        string.push_str(&format!("Maintainer: {}\n", self.maintainer));
        string.push_str(&format!("Architecture: {}\n", self.arch.to_string()));
        string.push_str(&format!("Version: {}\n", self.version));

        if let Some(ref homepage) = self.homepage {
            string.push_str(&format!("Homepage: {}\n", homepage));
        }

        string.push_str(&format!("Description: {}\n", self.short_desc));

        if let Some(ref long_desc) = self.long_desc {
            for line in long_desc.lines() {
                if line.len() == 0 {
                    string.push_str(" .\n");
                } else {
                    string.push_str(&format!(" {}\n", line));
                }
            }
        }

        string
    }

    /// Calculates the installed size of the package based on the input files.
    #[cfg(unix)]
    fn get_size(&self) -> u64 {
        let mut size = 0;

        for paths in &self.files {
            size += paths.0.metadata().unwrap().size();
        }

        (size / 1024) as u64
    }

    #[cfg(windows)]
    fn get_size(&self) -> u64 {
        0
    }
}


/// A builder object for sequentially creating a package.
pub struct PackageBuilder {
    name: Option<String>,
    files: Vec<(PathBuf, PathBuf)>,
    priority: Priority,
    section: String,
    depends: Vec<(String, String)>,
    maintainer: Option<String>,
    arch: Arch,
    version: Option<String>,
    homepage: Option<String>,
    short_desc: Option<String>,
    long_desc: Option<String>,
}

impl PackageBuilder {
    /// Creates a new package builder.
    pub fn new() -> PackageBuilder {
        PackageBuilder {
            name: None,
            files: Vec::new(),
            priority: Priority::Optional,
            section: "misc".to_string(),
            depends: Vec::new(),
            maintainer: None,
            arch: Arch::All,
            version: None,
            homepage: None,
            short_desc: None,
            long_desc: None,
        }
    }

    /// Attempts to build a package object from the current data.
    ///
    /// If the required fields have not been set, an error will be returned.
    pub fn build(self) -> Result<Package, Error> {
        if self.name.is_none() {
            return Err(Error::OptionMissing("the package name must be specified".to_string()));
        }

        if self.maintainer.is_none() {
            return Err(Error::OptionMissing("the package maintainer must be specified".to_string()));
        }

        if self.version.is_none() {
            return Err(Error::OptionMissing("the package version must be specified".to_string()));
        }

        if self.short_desc.is_none() {
            return Err(Error::OptionMissing("the package short description must be specified".to_string()));
        }

        Ok(Package {
            name: self.name.unwrap(),
            files: self.files,
            priority: self.priority,
            section: self.section,
            depends: self.depends,
            maintainer: self.maintainer.unwrap(),
            arch: self.arch,
            version: self.version.unwrap(),
            homepage: self.homepage,
            short_desc: self.short_desc.unwrap(),
            long_desc: self.long_desc,
        })
    }

    /// Sets the package name.
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    /// Adds a file to the package.
    pub fn add_file(&mut self, source: &Path, destination: &Path) -> &mut Self {
        self.files.push((source.to_path_buf(), destination.to_path_buf()));
        self
    }

    /// Sets the package priority.
    pub fn priority(&mut self, priority: Priority) -> &mut Self {
        self.priority = priority;
        self
    }

    /// Sets the package section name.
    pub fn section(&mut self, section: &str) -> &mut Self {
        self.section = section.to_string();
        self
    }

    /// Adds a package dependency.
    pub fn add_depends(&mut self, depends: (&str, &str)) -> &mut Self {
        self.depends.push((depends.0.to_string(), depends.1.to_string()));
        self
    }

    /// Sets the package maintainer.
    pub fn maintainer(&mut self, maintainer: &str) -> &mut Self {
        self.maintainer = Some(maintainer.to_string());
        self
    }

    /// Sets the package target architecture.
    pub fn arch(&mut self, arch: Arch) -> &mut Self {
        self.arch = arch;
        self
    }

    /// Sets the package version.
    pub fn version(&mut self, version: &str) -> &mut Self {
        self.version = Some(version.to_string());
        self
    }

    /// Sets the package homepage.
    pub fn homepage(&mut self, homepage: &str) -> &mut Self {
        self.homepage = Some(homepage.to_string());
        self
    }

    /// Sets the package short description.
    pub fn short_desc(&mut self, short_desc: &str) -> &mut Self {
        self.short_desc = Some(short_desc.to_string());
        self
    }

    /// Sets the package long description.
    pub fn long_desc(&mut self, long_desc: &str) -> &mut Self {
        self.long_desc = Some(long_desc.to_string());
        self
    }
}
