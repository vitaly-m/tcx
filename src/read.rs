use std::io::BufRead;
use std::num::{ParseFloatError, ParseIntError};
use std::str::{FromStr, ParseBoolError};

use chrono::DateTime;
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

macro_rules! must_read_value_as {
    ($to: tt. $attr:tt, $r: tt, $b: tt, $ft:ty) => {{
        loop {
            match $r.read_event(&mut $b) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"Value" => must_read_text_as!($to.$attr, $r, $b, $ft),
                    _ => (),
                },
                Ok(Event::End(ref e)) => {
                    if e.name() == b"Value" {
                        break;
                    }
                }
                Err(e) => return Err(ReadError::XmlReadError(e)),
                _ => (),
            }
        }
    }};
}

macro_rules! opt_read_value_as {
    ($to: tt. $attr:tt, $r: tt, $b: tt, $ft:ty) => {{
        loop {
            match $r.read_event(&mut $b) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"Value" => opt_read_text_as!($to.$attr, $r, $b, $ft),
                    _ => (),
                },
                Ok(Event::End(ref e)) => {
                    if e.name() == b"Value" {
                        break;
                    }
                }
                Err(e) => return Err(ReadError::XmlReadError(e)),
                _ => (),
            }
        }
    }};
}

macro_rules! must_read_text_as {
    ($to: tt. $attr:tt, $r: tt, $b: tt, $ft: ty) => {{
        if let Ok(Event::Text(ref t)) = $r.read_event(&mut $b) {
            $to.$attr = <$ft>::from_str(&t.unescape_and_decode($r)?)?;
        }
    }};
}

macro_rules! opt_read_text_as {
    ($to: tt. $attr:tt, $r: tt, $b: tt, $ft: ty) => {{
        if let Ok(Event::Text(ref t)) = $r.read_event(&mut $b) {
            $to.$attr = Some(<$ft>::from_str(&t.unescape_and_decode($r)?)?);
        }
    }};
}

macro_rules! must_read_text {
    ($to: tt. $attr:tt, $r: tt, $b: tt) => {{
        if let Ok(Event::Text(ref t)) = $r.read_event(&mut $b) {
            $to.$attr = t.unescape_and_decode($r)?;
        }
    }};
}

macro_rules! opt_read_text {
    ($to: tt. $attr:tt, $r: tt, $b: tt) => {{
        if let Ok(Event::Text(ref t)) = $r.read_event(&mut $b) {
            $to.$attr = Some(t.unescape_and_decode($r)?);
        }
    }};
}

macro_rules! must_read_text_as_date {
    ($to: tt. $attr:tt, $r: tt, $b: tt) => {
        if let Ok(Event::Text(ref t)) = $r.read_event(&mut $b) {
            $to.$attr = chrono::DateTime::parse_from_rfc3339(t.unescape_and_decode(&$r)?.as_str())?;
        }
    };
}

macro_rules! opt_read_text_as_date {
    ($to: tt. $attr:tt, $r: tt, $b: tt) => {
        if let Ok(Event::Text(ref t)) = $r.read_event(&mut $b) {
            $to.$attr = Some(chrono::DateTime::parse_from_rfc3339(
                t.unescape_and_decode(&$r)?.as_str(),
            )?);
        }
    };
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
                    must_read_text_as_date!(activity.id, reader, buf);
                }
                b"Lap" => {
                    activity.laps.push(read_activity_lap(reader, b"Lap", e)?);
                }
                b"Notes" => {
                    opt_read_text!(activity.notes, reader, buf);
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
                    must_read_text_as!(activity.sport, reader, buf, Sport);
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

fn read_activity_lap<B: BufRead>(
    reader: &mut Reader<B>,
    close_tag: &[u8],
    lap_element: &BytesStart,
) -> Result<ActivityLap, ReadError> {
    let mut buf = Vec::new();
    let mut a_lap = ActivityLap::default();
    for ar in lap_element.attributes() {
        if let Ok(a) = ar {
            match a.key {
                b"StartTime" => {
                    a_lap.start_time =
                        DateTime::parse_from_rfc3339(&a.unescape_and_decode_value(reader)?)?;
                }
                _ => (),
            }
        }
    }
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"TotalTimeSeconds" => {
                    must_read_text_as!(a_lap.total_time_seconds, reader, buf, f64);
                }
                b"DistanceMeters" => {
                    must_read_text_as!(a_lap.distance_meters, reader, buf, f64);
                }
                b"MaximumSpeed" => {
                    opt_read_text_as!(a_lap.maximum_speed, reader, buf, f64);
                }
                b"Calories" => {
                    must_read_text_as!(a_lap.calories, reader, buf, u16);
                }
                b"AverageHeartRateBpm" => {
                    opt_read_value_as!(a_lap.average_heart_rate_bpm, reader, buf, u8);
                }
                b"MaximumHeartRateBpm" => {
                    opt_read_value_as!(a_lap.maximum_heart_rate_bpm, reader, buf, u8);
                }
                b"Intensity" => {
                    must_read_text_as!(a_lap.intensity, reader, buf, Intensity);
                }
                b"Cadence" => {
                    opt_read_text_as!(a_lap.cadence, reader, buf, u8);
                }
                b"TriggerMethod" => {
                    must_read_text_as!(a_lap.trigger_method, reader, buf, TriggerMethod);
                }
                b"Track" => {
                    a_lap.track_points = read_track(reader, b"Track")?;
                }
                b"Notes" => {
                    opt_read_text!(a_lap.notes, reader, buf);
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
    Ok(a_lap)
}

fn read_track<B: BufRead>(
    reader: &mut Reader<B>,
    close_tag: &[u8],
) -> Result<Vec<TrackPoint>, ReadError> {
    let mut buf = Vec::new();
    let mut track = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Trackpoint" => track.push(read_track_point(reader, b"Trackpoint")?),
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
    Ok(track)
}

fn read_track_point<B: BufRead>(
    reader: &mut Reader<B>,
    close_tag: &[u8],
) -> Result<TrackPoint, ReadError> {
    let mut buf = Vec::new();
    let mut tp = TrackPoint::default();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Time" => {
                    must_read_text_as_date!(tp.time, reader, buf);
                }
                b"Position" => {
                    tp.position = Some(read_position(reader, b"Position")?);
                }
                b"AltitudeMeters" => {
                    opt_read_text_as!(tp.altitude_meters, reader, buf, f64);
                }
                b"DistanceMeters" => {
                    opt_read_text_as!(tp.distance_meters, reader, buf, f64);
                }
                b"HeartRateBpm" => {
                    opt_read_value_as!(tp.heart_rate_bpm, reader, buf, u8);
                }
                b"Cadence" => {
                    opt_read_text_as!(tp.cadence, reader, buf, u8);
                }
                b"SensorState" => {
                    opt_read_text_as!(tp.sensor_state, reader, buf, SensorState);
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
    Ok(tp)
}

fn read_position<B: BufRead>(
    reader: &mut Reader<B>,
    close_tag: &[u8],
) -> Result<Position, ReadError> {
    let mut buf = Vec::new();
    let mut pos = Position::default();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"LatitudeDegrees" => {
                    must_read_text_as!(pos.latitude_degrees, reader, buf, f64);
                }
                b"LongitudeDegrees" => {
                    must_read_text_as!(pos.longitude_degrees, reader, buf, f64);
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
    Ok(pos)
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
                    opt_read_text!(plan.name, reader, buf);
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
                    must_read_text_as!(quick_workout.total_time_seconds, reader, buf, f64);
                }
                b"DistanceMeters" => {
                    must_read_text_as!(quick_workout.distance_meters, reader, buf, f64);
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
                    must_read_text!(d.name, reader, buf);
                }
                b"UnitId" => {
                    must_read_text_as!(d.unit_id, reader, buf, u32);
                }
                b"ProductID" => {
                    must_read_text_as!(d.product_id, reader, buf, u16);
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
                    must_read_text!(a.name, reader, buf);
                }
                b"Build" => a.build = read_build(reader)?,
                b"LangID" => {
                    must_read_text!(a.lang_id, reader, buf);
                }
                b"PartNumber" => {
                    must_read_text!(a.part_number, reader, buf);
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
                    opt_read_text!(build.time, reader, buf);
                }
                b"Build" => {
                    opt_read_text!(build.builder, reader, buf);
                }
                b"Type" => {
                    opt_read_text_as!(build.build_type, reader, buf, BuildType);
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
                    must_read_text_as!(version.version_major, reader, buf, u16);
                }
                b"VersionMinor" => {
                    must_read_text_as!(version.version_minor, reader, buf, u16);
                }
                b"BuildMajor" => {
                    opt_read_text_as!(version.build_major, reader, buf, u16);
                }
                b"BuildMinor" => {
                    opt_read_text_as!(version.build_minor, reader, buf, u16);
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
        assert_eq!(
            Training {
                quick_workout_results: None,
                plan: Some(Plan {
                    interval_workout: false,
                    training_type: TrainingType::Workout,
                    name: None,
                }),
                virtual_partner: false,
            },
            activity.training.unwrap()
        );
        assert_eq!(10, activity.laps.len());
    }

    #[test]
    fn read_activity_lap_test() {
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
        let lap = activity.laps.into_iter().next().unwrap();
        assert_eq!(1000.0, f64::from_str("1000.0").unwrap());
        assert_eq!(525.0, lap.total_time_seconds);
        assert_eq!(1000.0, lap.distance_meters);
        assert_eq!(Some(2.330555650922987), lap.maximum_speed);
        assert_eq!(779, lap.calories);
        assert_eq!(Some(127), lap.average_heart_rate_bpm);
        assert_eq!(Some(137), lap.maximum_heart_rate_bpm);
        assert_eq!(Intensity::Active, lap.intensity);
        assert_eq!(Some(90), lap.cadence);
        assert_eq!(TriggerMethod::Distance, lap.trigger_method);
        assert_eq!(true, lap.validate().is_ok());
        assert_eq!(525, lap.track_points.len());
    }

    #[test]
    fn read_track_point_test() {
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
        let tp = activity
            .laps
            .into_iter()
            .next()
            .unwrap()
            .track_points
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(
            FixedOffset::east(0)
                .ymd(2020, 12, 28)
                .and_hms_milli(13, 36, 17, 453),
            tp.time
        );
        assert_eq!(
            Some(Position {
                latitude_degrees: 51.752415,
                longitude_degrees: 39.18763,
            }),
            tp.position
        );
        assert_eq!(Some(178.615), tp.altitude_meters);
        assert_eq!(Some(0.0), tp.distance_meters);
        assert_eq!(Some(68), tp.heart_rate_bpm);
        assert_eq!(Some(0), tp.cadence);
        assert_eq!(Some(SensorState::Present), tp.sensor_state);
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
