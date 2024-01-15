use std::{str::FromStr, sync::Mutex};

use anyhow::Result;
use chrono::{DateTime, Utc};
use lenient_semver::parse;
use semver::Version;

#[derive(Debug, PartialEq, Clone)]
pub enum Dist {
    Ubuntu(Option<Version>),
    Debian(Option<Version>),
    Fedora(Option<Version>),
}

#[derive(Debug, PartialEq)]
pub enum Arch {
    Amd64,
}

impl FromStr for Arch {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "amd64" => Ok(Arch::Amd64),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Type {
    Deb,
    Rpm,
}

#[derive(Debug)]
pub struct Package {
    tipe: Type,
    pub(crate) dist: Option<Dist>,
    url: String,
    ver: String,
    data: Mutex<Option<Vec<u8>>>,
    created: DateTime<Utc>,
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.tipe == other.tipe
            && self.dist == other.dist
            && self.url == other.url
            && self.ver == other.ver
            && *self.data.lock().unwrap() == *other.data.lock().unwrap()
            && self.created == other.created
    }
}

struct DetectError;

impl Package {
    pub fn detect_package(
        name: &str,
        ver: String,
        url: String,
        created: DateTime<Utc>,
    ) -> Result<Package, ()> {
        // Split the extension first.
        // If we don't recognize it, then return error.
        let Some((tipe, splitted)) = split_extention(name) else {
            return Err(());
        };

        let mut dist: Option<Dist> = None;
        let sections: Vec<&str> = splitted.split(['-', '_']).collect();

        for section in sections {
            match section {
                dst if dst.contains("ubuntu") => dist = Some(Dist::Ubuntu(parse_version(dst))),
                dst if dst.contains("debian") => dist = Some(Dist::Debian(parse_version(dst))),
                dst if dst.contains("fedora") => dist = Some(Dist::Fedora(parse_version(dst))),
                _ => (),
            }
        }

        Ok(Package {
            tipe,
            dist,
            url,
            ver,
            data: Mutex::new(None),
            created,
        })
    }

    pub fn is_deb(&self) -> bool {
        self.tipe == Type::Deb
    }

    /// Check if the package is for Ubuntu
    pub fn for_ubuntu(&self) -> bool {
        matches!(self.dist, Some(Dist::Ubuntu(_)))
    }

    /// Return the distribution for which it was packaged
    pub fn distribution(&self) -> &Dist {
        self.dist.as_ref().unwrap()
    }

    /// Version of the package
    pub fn version(&self) -> &str {
        &self.ver
    }

    pub fn download_url(&self) -> &str {
        &self.url
    }

    pub fn file_name(&self) -> &str {
        &self.url.split('/').last().unwrap()
    }

    /// Download package data
    ///
    /// It is required to call this function before calling the `data()` function.
    pub async fn download(&self) -> Result<()> {
        let data = reqwest::get(self.download_url()).await?.bytes().await?;
        *self.data.lock().unwrap() = Some(data.to_vec());
        Ok(())
    }

    /// Return the data of the package.
    ///
    /// It is required to call the `download()` function before calling this.
    /// Otherwise, `None` is returned.
    pub fn data(&self) -> Option<Vec<u8>> {
        self.data.lock().unwrap().clone()
    }

    #[cfg(test)]
    /// Set the internal package data.
    ///
    /// It's for testing purpose.
    pub fn set_data(&self, data: Vec<u8>) {
        *self.data.lock().unwrap() = Some(data);
    }

    pub fn creation_date(&self) -> &DateTime<Utc> {
        &self.created
    }
}

/// Parses the version from the distribution identifier `dist`.
///
/// For instance, for a distribution identifier `ubuntu22.10` it will
/// parse the version as `22.10`.
fn parse_version(dist: &str) -> Option<Version> {
    parse(split_at_numeric(dist)?).ok()
}

/// Splits the string `s` at the first occurence of a numeric digit.
///
/// It is used to extract version number from strings, such as for "ubuntu24.10" it would
/// return "24.10".
fn split_at_numeric(s: &str) -> Option<&str> {
    for (curr, (index, next)) in s.chars().zip(s.char_indices().skip(1)) {
        if curr.is_ascii_alphabetic() && next.is_ascii_digit() {
            return Some(&s[index..]);
        }
    }

    None
}

fn split_extention(s: &str) -> Option<(Type, &str)> {
    let mut str = String::with_capacity(3);
    let mut index = 0;

    for (idx, ch) in s.char_indices().rev() {
        if ch == '.' {
            index = idx;
            break;
        } else {
            str.push(ch);
        }
    }

    if index == 0 {
        return None;
    }

    let splitted = &s[0..index];

    // `str` is in reverse order, so we try to match it reversely.
    let tipe = match str.as_str() {
        "bed" => Type::Deb,
        "mpr" => Type::Rpm,
        _ => return None,
    };

    Some((tipe, splitted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package() {
        let pack = Package::detect_package(
            "OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb",
            "2.0.0".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(pack.version(), "2.0.0");
        assert_eq!(pack.dist, Some(Dist::Ubuntu(Some(parse("22.04").unwrap()))));
        assert_eq!(pack.tipe, Type::Deb);

        let pack = Package::detect_package(
            "OpenBangla-Keyboard_2.0.0-fedora36.rpm",
            "2.0.0".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(pack.version(), "2.0.0");
        assert_eq!(pack.dist, Some(Dist::Fedora(Some(parse("36").unwrap()))));
        assert_eq!(pack.tipe, Type::Rpm);

        let pack = Package::detect_package(
            "caprine_2.56.1_amd64.deb",
            "v2.56.1".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(pack.version(), "v2.56.1");
        assert_eq!(pack.dist, None);
        assert_eq!(pack.tipe, Type::Deb);
    }

    #[test]
    fn test_split_extension() {
        assert_eq!(
            split_extention("OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb"),
            Some((Type::Deb, "OpenBangla-Keyboard_2.0.0-ubuntu22.04"))
        );
        assert_eq!(
            split_extention("OpenBangla-Keyboard_2.0.0-fedora36.rpm"),
            Some((Type::Rpm, "OpenBangla-Keyboard_2.0.0-fedora36"))
        );
        assert_eq!(split_extention("caprine_2.56.1_amd64.snap"), None);
        assert_eq!(split_extention("deb"), None);
    }

    #[test]
    fn test_split_test() {
        assert_eq!(split_at_numeric("ubuntu24.10"), Some("24.10"));
        assert_eq!(split_at_numeric("ubuntu"), None);
    }

    #[test]
    fn test_parse_version() {
        assert_eq!(
            parse_version("ubuntu22.10").unwrap(),
            Version::new(22, 10, 0)
        );
        assert_eq!(parse_version("fedora37").unwrap(), Version::new(37, 0, 0));
    }
}
