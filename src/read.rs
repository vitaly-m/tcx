use std::io::BufRead;
use std::num::ParseIntError;
use std::str::FromStr;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use thiserror::Error;
use validator::Validate;

use crate::types::*;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("error reading XML '{0}'")]
    XmlReadError(#[from] quick_xml::Error),
    #[error("error while parsing to int '{0}'")]
    ParseIntError(#[from] ParseIntError),
    #[error("type not defined, but expected")]
    TypeNotDefined,
    #[error("unknown '{0}' value '{1}'")]
    UnknownEnumValue(&'static str, String),
}

fn read_training_center<B: BufRead>(
    reader: &mut Reader<B>,
) -> Result<TrainingCenterDatabase, ReadError> {
    let mut buf = Vec::new();
    let mut tc_db = TrainingCenterDatabase {
        folders: None,
        activity_list: None,
        workout_list: None,
        course_list: None,
        author: None,
    };
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Author" => {
                    let e_type = read_type(reader, e)?;
                    if e_type.as_str() == "Application_t" {
                        tc_db.author = Some(SourceType::Application(read_application(
                            reader, b"Author",
                        )?));
                    } else if e_type.as_str() == "Device_t" {
                        tc_db.author = Some(SourceType::Device(read_device(reader, b"Author")?));
                    }
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => match e.name() {
                b"TrainingCenterDatabase" => break,
                _ => {}
            },
            Err(e) => return Err(ReadError::XmlReadError(e)),
            _ => (),
        }
    }
    Ok(tc_db)
}

fn read_type<B: BufRead>(reader: &Reader<B>, e: &BytesStart) -> Result<String, ReadError> {
    match e
        .attributes()
        .filter(|a| a.is_ok() && a.as_ref().unwrap().key == b"xsi:type")
        .next()
    {
        None => Err(ReadError::TypeNotDefined),
        Some(ar) => Ok(ar?.unescape_and_decode_value(reader)?),
    }
}

fn read_device<B: BufRead>(reader: &mut Reader<B>, close_tag: &[u8]) -> Result<Device, ReadError> {
    let mut buf = Vec::new();
    let mut d = Device::default();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Name" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        d.name = t.unescape_and_decode(reader)?;
                    }
                }
                b"UnitId" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        d.unit_id = u32::from_str(&t.unescape_and_decode(reader)?)?;
                    }
                }
                b"ProductID" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        d.product_id = u16::from_str(&t.unescape_and_decode(reader)?)?;
                    }
                }
                b"Version" => {
                    d.version = read_version(reader)?;
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => {
                if e.name() == close_tag {
                    break;
                }
            }
            Err(e) => return Err(ReadError::XmlReadError(e)),
            _ => (),
        }
    }
    Ok(d)
}

fn read_application<B: BufRead>(
    reader: &mut Reader<B>,
    close_tag: &[u8],
) -> Result<Application, ReadError> {
    let mut buf = Vec::new();
    let mut a = Application::default();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Name" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        a.name = t.unescape_and_decode(&reader)?;
                    }
                }
                b"Build" => a.build = read_build(reader)?,
                b"LangID" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        a.lang_id = t.unescape_and_decode(&reader)?;
                    }
                }
                b"PartNumber" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        a.part_number = t.unescape_and_decode(&reader)?;
                    }
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => {
                if e.name() == close_tag {
                    break;
                }
            }
            Err(e) => {
                return Err(ReadError::XmlReadError(e));
            }
            _ => (),
        }
        buf.clear();
    }
    return Ok(a);
}

fn read_build<B: BufRead>(reader: &mut Reader<B>) -> Result<Build, ReadError> {
    let mut buf = Vec::new();
    let mut build = Build::default();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Version" => build.version = read_version(reader)?,
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
                        match t.unescape_and_decode(&reader)?.as_str() {
                            "Internal" => build.build_type = Some(BuildType::Internal),
                            "Alpha" => build.build_type = Some(BuildType::Alpha),
                            "Beta" => build.build_type = Some(BuildType::Beta),
                            "Release" => build.build_type = Some(BuildType::Release),
                            _ => {
                                return Err(ReadError::UnknownEnumValue(
                                    "build type",
                                    t.unescape_and_decode(&reader)?,
                                ));
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
            Err(e) => return Err(ReadError::XmlReadError(e)),
            _ => (),
        }
        buf.clear();
    }
    Ok(build)
}

fn read_version<B: BufRead>(reader: &mut Reader<B>) -> Result<Version, ReadError> {
    let mut buf = Vec::new();
    let mut version = Version::default();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"VersionMajor" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        version.version_major = u16::from_str(&t.unescape_and_decode(reader)?)?;
                    }
                }
                b"VersionMinor" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        version.version_minor = u16::from_str(&t.unescape_and_decode(reader)?)?;
                    }
                }
                b"BuildMajor" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        version.build_major = u16::from_str(&t.unescape_and_decode(reader)?).ok();
                    }
                }
                b"BuildMinor" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        version.build_minor = u16::from_str(&t.unescape_and_decode(reader)?).ok();
                    }
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match e.name() {
                b"Version" => break,
                _ => (),
            },
            Err(e) => return Err(ReadError::XmlReadError(e)),
            _ => (),
        }
        buf.clear();
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use validator::ValidationErrors;

    use super::*;

    #[test]
    fn read_device_test() {
        let tcx_bytes: &[u8] = include_bytes!("../test_resources/+__2020-12-28_16-36-16.TCX.xml");
        let mut reader = Reader::from_reader(tcx_bytes);
        let device = read_device(&mut reader, b"Creator").unwrap();
        assert_eq!(
            Device {
                name: String::from("Polar Vantage V"),
                unit_id: 0,
                product_id: 203,
                version: Version {
                    version_major: 5,
                    version_minor: 1,
                    build_major: Some(0),
                    build_minor: Some(0),
                },
            },
            device
        )
    }

    #[test]
    fn read_training_center_db_test() {
        let tcx_bytes: &[u8] = include_bytes!("../test_resources/+__2020-12-28_16-36-16.TCX.xml");
        let mut reader = Reader::from_reader(tcx_bytes);
        let tc = read_training_center(&mut reader).unwrap();
        assert_eq!(
            SourceType::Application(Application {
                name: "Polar Flow Mobile Viewer Android".to_string(),
                lang_id: "EN".to_string(),
                part_number: "XXX-XXXXX-XX".to_string(),
                build: Build {
                    version: Version {
                        version_major: 0,
                        version_minor: 0,
                        build_major: None,
                        build_minor: None,
                    },
                    build_type: None,
                    time: None,
                    builder: None,
                },
            }),
            tc.author.unwrap()
        )
    }

    #[test]
    fn test_application_validate() {
        let mut application = Application::default();
        let vr: ValidationErrors = application.validate().unwrap_err();
        assert_eq!(true, vr.field_errors().contains_key("part_number"));
        assert_eq!(true, vr.field_errors().contains_key("lang_id"));
        application.part_number = String::from("XXX-XXXXX-XX");
        application.lang_id = String::from("EN");
        assert_eq!(true, application.validate().is_ok())
    }
}
