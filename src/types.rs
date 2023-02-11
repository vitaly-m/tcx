use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::DateTime;
use chrono::{Utc};
use regex::Regex;
use thiserror::Error;
use validator::Validate;

#[derive(Error, Debug)]
pub enum UnknownEnumValueError {
    TrainingType(String),
    Sport(String),
    BuildType(String),
    Intensity(String),
    TriggerMethod(String),
    SensorState(String),
    CadenceSensorType(String),
}

impl Display for UnknownEnumValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnknownEnumValueError::TrainingType(t) => write!(f, "unknown '{}' training type", t),
            UnknownEnumValueError::Sport(t) => write!(f, "unknown '{}' sport", t),
            UnknownEnumValueError::BuildType(t) => write!(f, "unknown '{}' build type", t),
            UnknownEnumValueError::Intensity(t) => write!(f, "unknown '{}' intensity", t),
            UnknownEnumValueError::TriggerMethod(t) => write!(f, "unknown '{}' trigger method", t),
            UnknownEnumValueError::SensorState(t) => write!(f, "unknown '{}' sensor state", t),
            UnknownEnumValueError::CadenceSensorType(t) => {
                write!(f, "unknown '{}' cadence sensor type", t)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SourceType {
    Application(Application),
    Device(Device),
}

#[derive(Debug, PartialEq)]
pub enum BuildType {
    Internal,
    Alpha,
    Beta,
    Release,
}

impl FromStr for BuildType {
    type Err = UnknownEnumValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Internal" => Ok(BuildType::Internal),
            "Alpha" => Ok(BuildType::Alpha),
            "Beta" => Ok(BuildType::Beta),
            "Release" => Ok(BuildType::Release),
            _ => {
                return Err(UnknownEnumValueError::BuildType(s.to_string()));
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CoursePointType {
    Generic,
    Summit,
    Valley,
    Water,
    Food,
    Danger,
    Left,
    Right,
    Straight,
    FirstAid,
    Category4,
    Category3,
    Category2,
    Category1,
    HorsCategory,
    Sprint,
}

#[derive(Debug, PartialEq)]
pub enum StepType {
    Step(Step),
    Repeat(Repeat),
}

#[derive(Debug, PartialEq)]
pub enum Target {
    Speed(Zone),
    HeartRate(Zone),
    Cadence(Cadence),
    None,
}

#[derive(Debug, PartialEq)]
pub enum Zone {
    PredefinedSpeedZone(u8),
    CustomSpeedZone(CustomSpeedZone),
    PredefinedHeartRateZone(u8),
    CustomHeartRateZone(CustomHeartRateZone),
}

#[derive(Debug, PartialEq)]
pub enum SpeedType {
    Pace,
    Speed,
}

#[derive(Debug, PartialEq)]
pub enum Duration {
    Time(u16),
    Distance(u16),
    HeartRateAbove(u8),
    HeartRateBelow(u8),
    CaloriesBurned(u16),
}

#[derive(Debug, PartialEq)]
pub enum TrainingType {
    Workout,
    Course,
}

impl Default for TrainingType {
    fn default() -> Self {
        TrainingType::Workout
    }
}

impl FromStr for TrainingType {
    type Err = UnknownEnumValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Workout" => Ok(TrainingType::Workout),
            "Course" => Ok(TrainingType::Course),
            _ => {
                return Err(UnknownEnumValueError::TrainingType(s.to_string()));
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SensorState {
    Present,
    Absent,
}

impl Default for SensorState {
    fn default() -> Self {
        Self::Present
    }
}

impl FromStr for SensorState {
    type Err = UnknownEnumValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Present" => Ok(Self::Present),
            "Absent" => Ok(Self::Absent),
            _ => {
                return Err(UnknownEnumValueError::SensorState(s.to_string()));
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Intensity {
    Active,
    Resting,
}

impl FromStr for Intensity {
    type Err = UnknownEnumValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Active" => Ok(Self::Active),
            "Resting" => Ok(Self::Resting),
            _ => {
                return Err(UnknownEnumValueError::Intensity(s.to_string()));
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TriggerMethod {
    Manual,
    Distance,
    Location,
    Time,
    HeartRate,
}

impl FromStr for TriggerMethod {
    type Err = UnknownEnumValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Manual" => Ok(Self::Manual),
            "Distance" => Ok(Self::Distance),
            "Location" => Ok(Self::Location),
            "Time" => Ok(Self::Time),
            "HeartRate" => Ok(Self::HeartRate),
            _ => {
                return Err(UnknownEnumValueError::TriggerMethod(s.to_string()));
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Sport {
    Running,
    Biking,
    Other,
}

impl FromStr for Sport {
    type Err = UnknownEnumValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Running" => Ok(Sport::Running),
            "Biking" => Ok(Sport::Biking),
            "Other" => Ok(Sport::Other),
            _ => {
                return Err(UnknownEnumValueError::Sport(s.to_string()));
            }
        }
    }
}

lazy_static! {
    static ref PART_NUMBER_REGEX: Regex =
        Regex::new(r"[\p{Lu}\d]{3}-[\p{Lu}\d]{5}-[\p{Lu}\d]{2}").unwrap();
}

/// Identifies a PC software application.
#[derive(Default, Debug, PartialEq, Validate)]
pub struct Application {
    pub name: String,
    pub build: Build,
    /// Specifies the two character ISO 693-1 language id that identifies the installed
    /// language of this application. see http://www.loc.gov/standards/iso639-2/
    /// for appropriate ISO identifiers
    #[validate(length(equal = 2))]
    pub lang_id: String,
    /// The formatted XXX-XXXXX-XX Garmin part number of a PC application.
    #[validate(regex = "PART_NUMBER_REGEX")]
    pub part_number: String,
}

/// Information about the build.
#[derive(Default, Debug, PartialEq)]
pub struct Build {
    pub version: Version,
    pub build_type: Option<BuildType>,
    /// A string containing the date and time when an application was built.
    /// Note that this is not an xsd:dateTime type because this string is
    /// generated by the compiler and cannot be readily converted to the
    /// xsd:dateTime format.
    pub time: Option<String>,
    /// The login name of the engineer who created this build.
    pub builder: Option<String>,
}

#[derive(Default, Debug, PartialEq)]
pub struct Version {
    pub version_major: u16,
    pub version_minor: u16,
    pub build_major: Option<u16>,
    pub build_minor: Option<u16>,
}

/// Identifies the originating GPS device that tracked a run or
/// used to identify the type of device capable of handling
/// the data for loading.
#[derive(Default, Debug, PartialEq)]
pub struct Device {
    pub name: String,
    pub unit_id: u32,
    pub product_id: u16,
    pub version: Version,
}

#[derive(Debug, PartialEq)]
pub struct TrainingCenterDatabase {
    pub folders: Option<Folders>,
    pub activity_list: Option<ActivityList>,
    pub workout_list: Option<WorkoutList>,
    pub course_list: Option<CourseList>,
    pub author: Option<SourceType>,
}

#[derive(Debug, PartialEq)]
pub struct CourseList {
    pub cources: Option<Vec<Course>>,
}

#[derive(Debug, PartialEq)]
pub struct Course {
    pub name: Option<String>,
    pub laps: Option<Vec<CourseLap>>,
    pub track_points: Option<Vec<TrackPoint>>,
    pub notes: Option<String>,
    pub course_point: Option<CoursePoint>,
    pub creator: Option<SourceType>,
}

#[derive(Debug, PartialEq)]
pub struct CoursePoint {
    pub name: Option<String>,
    pub time: Option<DateTime<Utc>>,
    pub position: Option<Position>,
    pub altitude_meters: Option<f64>,
    pub point_type: Option<CoursePointType>,
    pub notes: Option<String>,
}

#[derive(Debug, PartialEq, Validate)]
pub struct CourseLap {
    pub total_time_seconds: Option<f64>,
    pub distance_meters: Option<f64>,
    pub begin_position: Option<Position>,
    pub begin_altitude_meters: Option<f64>,
    pub end_position: Option<Position>,
    pub end_altitude_meters: Option<f64>,
    pub average_heart_rate_bpm: Option<u8>,
    pub maximum_heart_rate_bpm: Option<u8>,
    pub intensity: Option<Intensity>,
    #[validate(range(max = 254))]
    pub cadence: Option<u8>,
}

#[derive(Debug, PartialEq)]
pub struct WorkoutList {
    pub workouts: Option<Vec<Workout>>,
}

#[derive(Debug, PartialEq)]
pub struct Workout {
    pub name: Option<String>,
    pub steps: Option<Vec<StepType>>,
    pub scheduled_on: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub creator: Option<SourceType>,
    pub sport: Option<Sport>,
}

#[derive(Debug, PartialEq)]
pub struct Repeat {
    pub step_id: Option<u8>,
    pub repetitions: Option<u8>,
    pub children: Option<Vec<StepType>>,
}

#[derive(Debug, PartialEq)]
pub struct Step {
    pub step_id: Option<u8>,
    pub name: Option<String>,
    pub duration: Option<Duration>,
    pub intensity: Option<Intensity>,
    pub target: Option<Target>,
}

#[derive(Debug, PartialEq)]
pub struct Cadence {
    pub low: Option<f64>,
    pub high: Option<f64>,
}

#[derive(Debug, PartialEq)]
pub struct CustomHeartRateZone {
    pub low: Option<u8>,
    pub high: Option<u8>,
}

#[derive(Debug, PartialEq)]
pub struct CustomSpeedZone {
    pub view_as: Option<SpeedType>,
    pub low_in_meters_per_second: Option<f64>,
    pub high_in_meters_per_second: Option<f64>,
}

#[derive(Debug, PartialEq, Default)]
pub struct ActivityList {
    pub activities: Vec<Activity>,
    pub multi_sport_sessions: Vec<MultiSportSession>,
}

#[derive(Debug, PartialEq)]
pub struct MultiSportSession {
    pub id: Option<DateTime<Utc>>,
    pub sports: Option<Vec<MultiActivity>>,
    pub notes: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct MultiActivity {
    pub transition: Option<ActivityLap>,
    pub activity: Option<Activity>,
}

#[derive(Debug, PartialEq)]
pub struct Folders {
    pub history: Option<History>,
    pub workouts: Option<Workouts>,
    pub courses: Option<Courses>,
}

#[derive(Debug, PartialEq)]
pub struct Courses {
    pub course_folder: Option<CourseFolder>,
}

#[derive(Debug, PartialEq)]
pub struct CourseFolder {
    pub folders: Option<Vec<CourseFolder>>,
    pub course_name_refs: Option<Vec<String>>,
    pub notes: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Workouts {
    pub running: Option<WorkoutFolder>,
    pub biking: Option<WorkoutFolder>,
    pub other: Option<WorkoutFolder>,
}

#[derive(Debug, PartialEq)]
pub struct WorkoutFolder {
    pub folders: Option<Vec<WorkoutFolder>>,
    pub workout_name_refs: Option<Vec<String>>,
    pub name: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct History {
    pub running: Option<HistoryFolder>,
    pub biking: Option<HistoryFolder>,
    pub other: Option<HistoryFolder>,
    pub multi_sport: Option<MultiSportFolder>,
}

#[derive(Debug, PartialEq)]
pub struct MultiSportFolder {
    pub folders: Option<Vec<MultiSportFolder>>,
    pub multisport_activity_refs: Option<Vec<DateTime<Utc>>>,
    pub weeks: Option<Vec<Week>>,
    pub notes: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct HistoryFolder {
    pub folders: Option<Vec<HistoryFolder>>,
    pub activity_refs: Option<Vec<DateTime<Utc>>>,
    pub weeks: Option<Vec<Week>>,
    pub notes: Option<String>,
    pub name: Option<String>,
}

/// The week is written out only if the notes are present.
#[derive(Debug, PartialEq)]
pub struct Week {
    pub notes: Option<String>,
    pub start_day: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq)]
pub struct Activity {
    pub id: DateTime<Utc>,
    pub laps: Vec<ActivityLap>,
    pub notes: Option<String>,
    pub training: Option<Training>,
    pub creator: Option<SourceType>,
    pub sport: Sport,
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            id: Utc::now(),
            laps: Vec::default(),
            notes: None,
            training: None,
            creator: None,
            sport: Sport::Running,
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Training {
    pub quick_workout_results: Option<QuickWorkout>,
    pub plan: Option<Plan>,
    pub virtual_partner: bool,
}

#[derive(Debug, PartialEq, Default, Validate)]
pub struct Plan {
    /// Non empty string up to 15 bytes
    #[validate(length(min = 1, max = 15))]
    pub name: Option<String>,
    pub training_type: TrainingType,
    pub interval_workout: bool,
}

#[derive(Debug, PartialEq, Default)]
pub struct QuickWorkout {
    pub total_time_seconds: f64,
    pub distance_meters: f64,
}

#[derive(Debug, PartialEq, Validate)]
pub struct ActivityLap {
    pub total_time_seconds: f64,
    pub distance_meters: f64,
    pub maximum_speed: Option<f64>,
    pub calories: u16,
    #[validate(range(min = 1))]
    pub average_heart_rate_bpm: Option<u8>,
    #[validate(range(min = 1))]
    pub maximum_heart_rate_bpm: Option<u8>,
    pub intensity: Intensity,
    #[validate(range(max = 254))]
    pub cadence: Option<u8>,
    pub trigger_method: TriggerMethod,
    pub track_points: Vec<TrackPoint>,
    pub notes: Option<String>,
    pub start_time: DateTime<Utc>,
    pub extension: Option<ActivityLapExtension>,
}

impl Default for ActivityLap {
    fn default() -> Self {
        Self {
            total_time_seconds: 0.0,
            distance_meters: 0.0,
            maximum_speed: None,
            calories: 0,
            average_heart_rate_bpm: None,
            maximum_heart_rate_bpm: None,
            intensity: Intensity::Active,
            cadence: None,
            trigger_method: TriggerMethod::Manual,
            track_points: Vec::default(),
            notes: None,
            start_time: Utc::now(),
            extension: None,
        }
    }
}

#[derive(Debug, PartialEq, Validate)]
pub struct TrackPoint {
    pub time: DateTime<Utc>,
    pub position: Option<Position>,
    pub altitude_meters: Option<f64>,
    pub distance_meters: Option<f64>,
    pub heart_rate_bpm: Option<u8>,
    #[validate(range(max = 254))]
    pub cadence: Option<u8>,
    pub sensor_state: Option<SensorState>,
    pub extension: Option<ActivityTrackPointExtension>,
}

impl Default for TrackPoint {
    fn default() -> Self {
        Self {
            time: Utc::now(),
            position: None,
            altitude_meters: None,
            distance_meters: None,
            heart_rate_bpm: None,
            cadence: None,
            sensor_state: None,
            extension: None,
        }
    }
}

#[derive(Debug, PartialEq, Default, Validate)]
pub struct Position {
    #[validate(range(min = - 90.0, max = 90.0))]
    pub latitude_degrees: f64,
    #[validate(range(min = - 180.0, max = 180.0))]
    pub longitude_degrees: f64,
}

// Activity Extensions

#[derive(Debug, PartialEq)]
pub enum CadenceSensorType {
    Footpod,
    Bike,
}

impl FromStr for CadenceSensorType {
    type Err = UnknownEnumValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Footpod" => Ok(Self::Footpod),
            "Bike" => Ok(Self::Bike),
            _ => {
                return Err(UnknownEnumValueError::CadenceSensorType(s.to_string()));
            }
        }
    }
}

#[derive(Debug, PartialEq, Default, Validate)]
pub struct ActivityTrackPointExtension {
    pub speed: Option<f64>,
    #[validate(range(max = 254))]
    pub run_cadence: Option<u8>,
    pub watts: Option<u16>,
    pub cadence_sensor: Option<CadenceSensorType>,
}

#[derive(Debug, PartialEq, Default, Validate)]
pub struct ActivityLapExtension {
    pub avg_speed: Option<f64>,
    #[validate(range(max = 254))]
    pub max_bike_cadence: Option<u8>,
    #[validate(range(max = 254))]
    pub avg_run_cadence: Option<u8>,
    #[validate(range(max = 254))]
    pub max_run_cadence: Option<u8>,
    pub steps: Option<u16>,
    pub avg_watts: Option<u16>,
    pub max_watts: Option<u16>,
}
