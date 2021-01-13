use std::io::BufRead;
use std::num::{ParseFloatError, ParseIntError};
use std::str::{FromStr, ParseBoolError};

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
    #[error("error while parsing to float '{0}'")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("error while parsing to bool '{0}'")]
    ParseBoolError(#[from] ParseBoolError),
    #[error("type not defined, but expected")]
    TypeNotDefined,
    #[error("error parsing enum value '{0}'")]
    UnknownEnumValue(#[from] UnknownEnumValueError),
    #[error("error while parsing to date '{0}'")]
    ParseDateError(#[from] chrono::ParseError),
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
                b"Activities" => {
                    tc_db.activity_list = Some(read_activity_list(reader, b"Activities")?)
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

fn read_activity_list<B: BufRead>(
    reader: &mut Reader<B>,
    close_tag: &[u8],
) -> Result<ActivityList, ReadError> {
    let mut buf = Vec::new();
    let mut al = ActivityList::default();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Activity" => al.activities.push(read_activity(reader, b"Activity")?),
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
        buf.clear();
    }
    Ok(al)
}

fn read_activity<B: BufRead>(
    reader: &mut Reader<B>,
    close_tag: &[u8],
) -> Result<Activity, ReadError> {
    let mut buf = Vec::new();
    let mut activity = Activity::default();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Id" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        activity.id = chrono::DateTime::parse_from_rfc3339(
                            t.unescape_and_decode(&reader)?.as_str(),
                        )?;
                    }
                }
                b"Lap" => {
                    //TODO read laps
                }
                b"Notes" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        activity.notes = t.unescape_and_decode(&reader).ok();
                    }
                }
                b"Training" => {
                    activity.training = Some(read_training(reader, b"Training")?);
                }
                b"Creator" => {
                    let e_type = read_type(reader, e)?;
                    if e_type.as_str() == "Application_t" {
                        activity.creator = Some(SourceType::Application(read_application(
                            reader, b"Creator",
                        )?));
                    } else if e_type.as_str() == "Device_t" {
                        activity.creator =
                            Some(SourceType::Device(read_device(reader, b"Creator")?));
                    }
                }
                b"Sport" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        activity.sport = Sport::from_str(&t.unescape_and_decode(&reader)?)?;
                    }
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
        buf.clear();
    }
    Ok(activity)
}

fn read_plan<B: BufRead>(
    reader: &mut Reader<B>,
    close_tag: &[u8],
    plan_element: &BytesStart,
) -> Result<Plan, ReadError> {
    let mut buf = Vec::new();
    let mut plan = Plan::default();
    for ar in plan_element.attributes() {
        if let Ok(a) = ar {
            match a.key {
                b"Type" => {
                    plan.training_type =
                        TrainingType::from_str(&a.unescape_and_decode_value(reader)?)?;
                }
                b"IntervalWorkout" => {
                    plan.interval_workout = bool::from_str(&a.unescape_and_decode_value(reader)?)?;
                }
                _ => (),
            }
        }
    }
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Name" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        plan.name = t.unescape_and_decode(reader).ok();
                    }
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
        buf.clear();
    }
    Ok(plan)
}

fn read_quick_workout<B: BufRead>(
    reader: &mut Reader<B>,
    close_tag: &[u8],
) -> Result<QuickWorkout, ReadError> {
    let mut buf = Vec::new();
    let mut quick_workout = QuickWorkout::default();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"TotalTimeSeconds" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        quick_workout.total_time_seconds =
                            f64::from_str(&t.unescape_and_decode(reader)?)?;
                    }
                }
                b"DistanceMeters" => {
                    if let Ok(Event::Text(ref t)) = reader.read_event(&mut buf) {
                        quick_workout.distance_meters =
                            f64::from_str(&t.unescape_and_decode(reader)?)?;
                    }
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
        buf.clear();
    }
    Ok(quick_workout)
}

fn read_training<B: BufRead>(
    reader: &mut Reader<B>,
    close_tag: &[u8],
) -> Result<Training, ReadError> {
    let mut buf = Vec::new();
    let mut training = Training::default();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"QuickWorkoutResults" => {
                    training.quick_workout_results =
                        Some(read_quick_workout(reader, b"QuickWorkoutResults")?);
                }
                b"Plan" => {
                    training.plan = Some(read_plan(reader, b"Plan", e)?);
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
        buf.clear();
    }
    Ok(training)
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
        buf.clear();
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
                        build.build_type =
                            Some(BuildType::from_str(&t.unescape_and_decode(&reader)?)?);
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
    use chrono::{FixedOffset, TimeZone};
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
    fn read_activities_test() {
        let tcx_bytes: &[u8] = include_bytes!("../test_resources/+__2020-12-28_16-36-16.TCX.xml");
        let mut reader = Reader::from_reader(tcx_bytes);
        let tc = read_training_center(&mut reader).unwrap();
        assert_eq!(1, tc.activity_list.as_ref().unwrap().activities.len());
        assert_eq!(
            0,
            tc.activity_list
                .as_ref()
                .unwrap()
                .multi_sport_sessions
                .len()
        );
    }

    #[test]
    fn read_activity_test() {
        let tcx_bytes: &[u8] = include_bytes!("../test_resources/+__2020-12-28_16-36-16.TCX.xml");
        let mut reader = Reader::from_reader(tcx_bytes);
        let tc = read_training_center(&mut reader).unwrap();
        let activity = tc
            .activity_list
            .unwrap()
            .activities
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(Sport::Running, activity.sport);
        assert_eq!(
            FixedOffset::east(0)
                .ymd(2020, 12, 28)
                .and_hms_milli(13, 36, 16, 453),
            activity.id
        );
        assert_eq!(
            SourceType::Device(Device {
                name: String::from("Polar Vantage V"),
                unit_id: 0,
                product_id: 203,
                version: Version {
                    version_major: 5,
                    version_minor: 1,
                    build_major: Some(0),
                    build_minor: Some(0),
                },
            }),
            activity.creator.unwrap()
        );
        assert_eq!(Training {
            quick_workout_results: None,
            plan: Some(Plan {
                interval_workout: false,
                training_type: TrainingType::Workout,
                name: None,
            }),
            virtual_partner: false,
        }, activity.training.unwrap());
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
