use error::*;
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::process::Command;
use tar::{Archive, Header};


pub enum Arch {
    X86,
    X64,
    All,
}

impl ToString for Arch {
    fn to_string(&self) -> String {
        match *self {
            Arch::X86 => "i386",
            Arch::X64 => "x86_64",
            Arch::All => "all",
        }.to_string()
    }
}


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
        }.to_string()
    }
}


pub struct Package {
    pub name: String,
    pub priority: Priority,
    pub section: String,
    pub size: u64,
    pub depends: Vec<(String, String)>,
    pub maintainer: String,
    pub arch: Arch,
    pub version: String,
    pub homepage: Option<String>,
    pub short_desc: String,
    pub long_desc: Option<String>,
}

impl Package {
    pub fn write_to(&self, file: &str) {
        let debian_file = self.create_debian_file();
        let control_archive = self.create_control_archive();
        let data_archive = self.create_data_archive();

        fs::remove_file(&file).unwrap_or(());

        Command::new("ar")
            .arg("rcD")
            .arg(&file)
            .arg(&debian_file)
            .arg(&control_archive)
            .arg(&data_archive)
            .output()
            .unwrap();

        fs::remove_file(&debian_file).unwrap();
        fs::remove_file(&control_archive).unwrap();
        fs::remove_file(&data_archive).unwrap();
    }

    fn create_debian_file(&self) -> String {
        let mut path = env::temp_dir();
        path.push("debian-binary");

        let mut file = File::create(&path).unwrap();
        file.write_all(b"2.0\n").unwrap();

        path.to_str().unwrap().to_string()
    }

    fn create_control_archive(&self) -> String {
        let mut path = env::temp_dir();
        path.push("control.tar");

        let file = File::create(&path).unwrap();
        let archive = Archive::new(file);

        let control_file = self.create_control_file();

        let mut header = Header::new();
        header.set_path("control").unwrap();
        header.set_size(control_file.len() as u64);
        header.set_mode(0644);
        header.set_cksum();

        archive.append(&header, &mut control_file.as_bytes()).unwrap();
        archive.finish().unwrap();

        path.to_str().unwrap().to_string()
    }

    fn create_data_archive(&self) -> String {
        let mut path = env::temp_dir();
        path.push("data.tar");

        let file = File::create(&path).unwrap();
        let archive = Archive::new(file);

        let control_file = self.create_control_file();

        let mut header = Header::new();
        header.set_path("control").unwrap();
        header.set_size(control_file.len() as u64);
        header.set_mode(0644);
        header.set_cksum();

        archive.append(&header, &mut control_file.as_bytes()).unwrap();
        archive.finish().unwrap();

        path.to_str().unwrap().to_string()
    }

    fn create_control_file(&self) -> String {
        let mut string = String::new();

        string.push_str(&format!("Package: {}\n", self.name));
        string.push_str(&format!("Priority: {}\n", self.priority.to_string()));
        string.push_str(&format!("Section: {}\n", self.section));
        string.push_str(&format!("Installed-Size: {}\n", self.size));

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
}


pub struct PackageBuilder {
    name: Option<String>,
    priority: Priority,
    section: String,
    size: Option<u64>,
    depends: Vec<(String, String)>,
    maintainer: Option<String>,
    arch: Arch,
    version: Option<String>,
    homepage: Option<String>,
    short_desc: Option<String>,
    long_desc: Option<String>,
}

impl PackageBuilder {
    pub fn new() -> PackageBuilder {
        PackageBuilder {
            name: None,
            priority: Priority::Optional,
            section: "misc".to_string(),
            size: None,
            depends: Vec::new(),
            maintainer: None,
            arch: Arch::All,
            version: None,
            homepage: None,
            short_desc: None,
            long_desc: None,
        }
    }

    pub fn build(self) -> Result<Package, Error> {
        if self.name.is_none() {
            return Err(Error::new_desc("the package name must be specified"));
        }

        if self.size.is_none() {
            return Err(Error::new_desc("the package size must be specified"));
        }

        if self.maintainer.is_none() {
            return Err(Error::new_desc("the package maintainer must be specified"));
        }

        if self.version.is_none() {
            return Err(Error::new_desc("the package version must be specified"));
        }

        if self.short_desc.is_none() {
            return Err(Error::new_desc("the package short description must be specified"));
        }

        Ok(Package {
            name: self.name.unwrap(),
            priority: self.priority,
            section: self.section,
            size: self.size.unwrap(),
            depends: self.depends,
            maintainer: self.maintainer.unwrap(),
            arch: self.arch,
            version: self.version.unwrap(),
            homepage: self.homepage,
            short_desc: self.short_desc.unwrap(),
            long_desc: self.long_desc,
        })
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn priority(&mut self, priority: Priority) -> &mut Self {
        self.priority = priority;
        self
    }

    pub fn section(&mut self, section: &str) -> &mut Self {
        self.section = section.to_string();
        self
    }

    pub fn size(&mut self, size: u64) -> &mut Self {
        self.size = Some(size);
        self
    }

    pub fn add_depends(&mut self, depends: (&str, &str)) -> &mut Self {
        self.depends.push((depends.0.to_string(), depends.1.to_string()));
        self
    }

    pub fn maintainer(&mut self, maintainer: &str) -> &mut Self {
        self.maintainer = Some(maintainer.to_string());
        self
    }

    pub fn arch(&mut self, arch: Arch) -> &mut Self {
        self.arch = arch;
        self
    }

    pub fn version(&mut self, version: &str) -> &mut Self {
        self.version = Some(version.to_string());
        self
    }

    pub fn homepage(&mut self, homepage: &str) -> &mut Self {
        self.homepage = Some(homepage.to_string());
        self
    }

    pub fn short_desc(&mut self, short_desc: &str) -> &mut Self {
        self.short_desc = Some(short_desc.to_string());
        self
    }

    pub fn long_desc(&mut self, long_desc: &str) -> &mut Self {
        self.long_desc = Some(long_desc.to_string());
        self
    }
}
