use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::types::BuildType::{Alpha, Beta, Internal, Release};

/// Identifies a PC software application.
#[derive(Debug, PartialEq)]
pub struct Application {
    pub name: Option<String>,
    pub build: Option<Build>,
    /// Specifies the two character ISO 693-1 language id that identifies the installed
    /// language of this application. see http://www.loc.gov/standards/iso639-2/
    /// for appropriate ISO identifiers
    pub lang_id: Option<String>,
    /// The formatted XXX-XXXXX-XX Garmin part number of a PC application.
    pub part_number: Option<String>,
}

impl Application {
    pub fn new() -> Self {
        return Self {
            name: None,
            build: None,
            lang_id: None,
            part_number: None,
        };
    }
}

/// Information about the build.
#[derive(Debug, PartialEq)]
pub struct Build {
    pub version: Option<Version>,
    pub build_type: Option<BuildType>,
    /// A string containing the date and time when an application was built.
    /// Note that this is not an xsd:dateTime type because this string is
    /// generated by the compiler and cannot be readily converted to the
    /// xsd:dateTime format.
    pub time: Option<String>,
    /// The login name of the engineer who created this build.
    pub builder: Option<String>,
}

impl Build {
    pub fn new() -> Self {
        Self {
            version: None,
            build_type: None,
            time: None,
            builder: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BuildType {
    Internal,
    Alpha,
    Beta,
    Release,
}

#[derive(Debug, PartialEq)]
pub struct Version {
    pub version_major: Option<u16>,
    pub version_minor: Option<u16>,
    pub build_major: Option<u16>,
    pub build_minor: Option<u16>,
}

impl Version {
    pub fn new() -> Self {
        Self {
            version_major: None,
            version_minor: None,
            build_major: None,
            build_minor: None,
        }
    }
}

/// Identifies the originating GPS device that tracked a run or
/// used to identify the type of device capable of handling
/// the data for loading.
#[derive(Debug, PartialEq)]
pub struct Device {
    pub name: Option<String>,
    pub unit_id: Option<u32>,
    pub product_id: Option<u16>,
    pub version: Option<Version>,
}

impl Device {
    pub fn new() -> Self {
        Self {
            name: None,
            unit_id: None,
            product_id: None,
            version: None,
        }
    }
}

//TODO support abstract source
fn read_device<B: BufRead>(reader: &mut Reader<B>) -> Result<Device, String> {
    let mut buf = Vec::new();
    let mut d = Device::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Name" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        d.name = t.unescape_and_decode(reader).ok();
                    }
                }
                b"UnitId" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        if let Ok(s) = t.unescape_and_decode(reader) {
                            d.unit_id = u32::from_str(&s).ok();
                        }
                    }
                }
                b"ProductID" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        if let Ok(s) = t.unescape_and_decode(reader) {
                            d.product_id = u16::from_str(&s).ok();
                        }
                    }
                }
                b"Version" => {
                    d.version = Some(read_version(reader)?);
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match e.name() {
                b"Creator" => break,
                _ => (),
            },
            Err(e) => {
                return Err(format!(
                    "Error parsing Device at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => (),
        }
    }
    Ok(d)
}

//TODO support abstract source
fn read_application<B: BufRead>(reader: &mut Reader<B>) -> Result<Application, String> {
    let mut buf = Vec::new();
    let mut a = Application::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Name" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        a.name = t.unescape_and_decode(&reader).ok();
                    }
                }
                b"Build" => a.build = Some(read_build(reader)?),
                b"LangID" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        a.lang_id = t.unescape_and_decode(&reader).ok();
                    }
                }
                b"PartNumber" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        a.part_number = t.unescape_and_decode(&reader).ok();
                    }
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match e.name() {
                b"Author" => break,
                _ => (),
            },
            Err(e) => {
                return Err(format!(
                    "Error parsing Application at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => (),
        }
        buf.clear();
    }
    return Ok(a);
}

fn read_build<B: BufRead>(reader: &mut Reader<B>) -> Result<Build, String> {
    let mut buf = Vec::new();
    let mut build = Build::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Version" => build.version = Some(read_version(reader)?),
                b"Time" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        build.time = t.unescape_and_decode(&reader).ok();
                    }
                }
                b"Build" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        build.builder = t.unescape_and_decode(&reader).ok();
                    }
                }
                b"Type" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        if let Ok(b_type) = t.unescape_and_decode(&reader) {
                            match b_type.as_str() {
                                "Internal" => build.build_type = Some(Internal),
                                "Alpha" => build.build_type = Some(Alpha),
                                "Beta" => build.build_type = Some(Beta),
                                "Release" => build.build_type = Some(Release),
                                _ => return Err(format!("Unknown build type {}", b_type)),
                            }
                        }
                    }
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match e.name() {
                b"Build" => break,
                _ => (),
            },
            Err(e) => {
                return Err(format!(
                    "Error parsing Build at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => (),
        }
        buf.clear();
    }
    Ok(build)
}

fn read_version<B: BufRead>(reader: &mut Reader<B>) -> Result<Version, String> {
    let mut buf = Vec::new();
    let mut version = Version::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"VersionMajor" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        if let Ok(s) = t.unescape_and_decode(reader) {
                            version.version_major = u16::from_str(&s).ok();
                        }
                    }
                }
                b"VersionMinor" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        if let Ok(s) = t.unescape_and_decode(reader) {
                            version.version_minor = u16::from_str(&s).ok();
                        }
                    }
                }
                b"BuildMajor" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        if let Ok(s) = t.unescape_and_decode(reader) {
                            version.build_major = u16::from_str(&s).ok();
                        }
                    }
                }
                b"BuildMinor" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        if let Ok(s) = t.unescape_and_decode(reader) {
                            version.build_minor = u16::from_str(&s).ok();
                        }
                    }
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match e.name() {
                b"Version" => break,
                _ => (),
            },
            Err(e) => {
                return Err(format!(
                    "Error parsing Version at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => (),
        }
        buf.clear();
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_application_test() {
        let tcx_bytes: &[u8] = include_bytes!("../test_resources/+__2020-12-28_16-36-16.TCX.xml");
        let mut reader = Reader::from_reader(tcx_bytes);
        let application = read_application(&mut reader);
        assert_eq!(
            Ok(Application {
                name: Some("Polar Flow Mobile Viewer Android".to_string()),
                lang_id: Some("EN".to_string()),
                part_number: Some("XXX-XXXXX-XX".to_string()),
                build: Some(Build {
                    version: Some(Version {
                        version_major: Some(0),
                        version_minor: Some(0),
                        build_major: None,
                        build_minor: None,
                    }),
                    build_type: None,
                    time: None,
                    builder: None
                })
            }),
            application
        )
    }

    #[test]
    fn read_device_test() {
        let tcx_bytes: &[u8] = include_bytes!("../test_resources/+__2020-12-28_16-36-16.TCX.xml");
        let mut reader = Reader::from_reader(tcx_bytes);
        let device = read_device(&mut reader);
        assert_eq!(
            Ok(Device {
                name: Some(String::from("Polar Vantage V")),
                unit_id: Some(0),
                product_id: Some(203),
                version: Some(Version {
                    version_major: Some(5),
                    version_minor: Some(1),
                    build_major: Some(0),
                    build_minor: Some(0),
                }),
            }),
            device
        )
    }
}
