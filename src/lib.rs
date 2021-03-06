//! Common types for `ruetian` Unbusy plugin.

#![deny(nonstandard_style, unused, future_incompatible, missing_docs)]
#![feature(range_is_empty)]
#![feature(map_first_last)]

use chrono::prelude::*;
use derive_more::Display;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::{cmp::*, convert::TryFrom, str::FromStr};

#[allow(missing_docs)]
pub mod errors {
    error_chain::error_chain! {
        foreign_links {
            NumParse(::std::num::ParseIntError);
        }
    }
}

use errors::*;

/// Department of a RUETian.
#[derive(
    Serialize, Deserialize, Debug, Hash, Clone, Copy, Eq, PartialEq, TryFromPrimitive, Display,
)]
#[repr(u32)]
#[allow(missing_docs)]
pub enum Department {
    CE = 0,
    EEE = 1,
    ME = 2,
    CSE = 3,
    ETE = 4,
    IPE = 5,
    GCE = 6,
    URP = 7,
    MTE = 8,
    Arch = 9,
    ECE = 10,
    CFPE = 11,
    BECM = 12,
    MSE = 13,

    Chem = 100,
    Math,
    Phy,
    Hum,
}

impl Department {
    /// Get official and colloquial name of a course.
    pub fn get_course_name(self, code: &str) -> Result<(&'static str, &'static str)> {
        use Department::*;
        match self {
            EEE => match code {
                "EEE 2100" => Ok(("Electrical Shop Practice", "Electrical Shop")),
                invalid => Err(format!("No course '{}' available for {}", invalid, self).into()),
            },
            _ => Err(format!("No course available for {}", self).into()),
        }
    }
}

/// Section of a RUETian.
#[allow(missing_docs)]
#[derive(Serialize, Deserialize, Debug, Hash, Clone, Copy, Eq, PartialEq, Display)]
pub enum Section {
    A,
    B,
    C,
}

/// Thirty of a RUETian.
#[derive(Serialize, Deserialize, Debug, Hash, Clone, Copy, Eq, PartialEq, Default)]
pub struct Thirty(pub u8);

/// Roll of a RUETian.
#[derive(Serialize, Deserialize, Debug, Hash, Clone, Copy, Eq, PartialEq, Display)]
pub struct Roll(u32);

#[allow(dead_code)]
impl Roll {
    /// Create a new valid instance of roll.
    #[inline]
    pub fn new(roll: u32) -> Result<Roll> {
        if roll < 10_000_000
            && Department::try_from((roll / 1000) % 100).is_ok()
            && roll % 1000 <= 180
        {
            Ok(Roll(roll))
        } else {
            Err(format!("Invalid roll: {}", roll).into())
        }
    }

    /// Get department of a RUETian.
    #[inline]
    pub fn department(self) -> Department {
        Department::try_from((self.0 / 1000) % 100).unwrap()
    }

    /// Get series of a RUETian.
    #[inline]
    pub fn series(self) -> u32 {
        self.0 / 100_000
    }

    /// Get roll in department of a RUETian.
    #[inline]
    pub fn roll_in_dept(self) -> u32 {
        self.0 % 1000
    }

    /// Get section of a RUETian.
    #[inline]
    pub fn section(self) -> Section {
        match self.roll_in_dept() {
            1..=60 => Section::A,
            61..=120 => Section::B,
            121..=180 => Section::C,
            other => panic!("Invalid roll in department: {}", other),
        }
    }

    /// Get thirty of a RUETian.
    #[inline]
    pub fn thirty(self) -> Thirty {
        match self.roll_in_dept() {
            1..=30 | 61..=90 | 121..=150 => Thirty(1),
            31..=60 | 91..=120 | 151..=180 => Thirty(2),
            other => panic!("Invalid roll in department: {}", other),
        }
    }
}

impl FromStr for Roll {
    type Err = Error;

    fn from_str(roll: &str) -> Result<Self> {
        let roll: u32 = roll.parse()?;
        Roll::new(roll)
    }
}

/// Weekday in the life of a RUETian.
#[allow(missing_docs)]
#[derive(Serialize, Deserialize, Debug, Hash, Clone, Copy, Eq, PartialEq)]
pub enum Day {
    A,
    B,
    C,
    D,
    E,
}

impl Day {
    /// Mutates `self` to be the next day and returns a copy.
    #[inline]
    pub fn succ_mut(&mut self) -> Day {
        *self = self.succ();
        *self
    }

    /// Returns the next day w/o mutating.
    #[inline]
    pub fn succ(self) -> Day {
        use Day::*;
        match self {
            A => B,
            B => C,
            C => D,
            D => E,
            E => A,
        }
    }
}

/// How frequently the class would gather.
#[derive(Serialize, Deserialize, Debug, Hash, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ClassFrequency {
    /// All sixty students of a section would gather every cycle.
    EveryCycleWithAll,

    /// The `Thirty` mentioned of a section would gather every cycle.
    EveryCycleWith(Thirty),

    /// All sixty students of a section would gather every odd cycles.
    OddCyclesWithAll,

    /// All sixty students of a section would gather every even cycles.
    EvenCyclesWithAll,

    /// The `Thirty` mentioned of a section would gather every odd cycle.
    /// The other `Thirty` would gather in even cycles.
    OddCyclesWith(Thirty),
}

impl Default for ClassFrequency {
    fn default() -> ClassFrequency {
        ClassFrequency::EveryCycleWithAll
    }
}

/// Describes a class on routine.
#[derive(Serialize, Deserialize, Debug, Hash, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClassInRoutine {
    /// Course for which the class is.
    pub course: String,

    /// Who would teach.
    pub teacher: String,

    /// Period number.
    pub period: u8,

    /// Where the class would sit.
    pub class_room: String,

    /// How long the class would run.
    #[serde(default = "ClassInRoutine::default_contact_hour")]
    pub contact_hours: u8,

    /// How frequently the class would gather.
    #[serde(default)]
    pub frequency: ClassFrequency,

    /// Any extra info (like the topic that would be discussed).
    #[serde(default)]
    pub comment: String,
}

impl ClassInRoutine {
    fn default_contact_hour() -> u8 {
        1
    }

    /// Check if the class would sit for a `roll` on a specific `cycle`.
    pub fn would_sit_for(&self, roll: Roll, cycle: u8) -> bool {
        use ClassFrequency::*;
        match self.frequency {
            EveryCycleWithAll => true,
            EveryCycleWith(thirty) if thirty == roll.thirty() => true,
            OddCyclesWithAll if cycle % 2 != 0 => true,
            EvenCyclesWithAll if cycle % 2 == 0 => true,
            OddCyclesWith(thirty) if cycle % 2 != 0 && thirty == roll.thirty() => true,
            _ => false,
        }
    }
}

/// A class routine.
pub type ClassRoutine = std::collections::HashMap<Day, Vec<ClassInRoutine>>;

/// Peoples scope for whom classes to be off.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct WhoScope {
    pub section: Option<Section>,
    pub thirty: Thirty,
}

impl WhoScope {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default(&self) -> bool {
        self.section == None && self.thirty == Thirty(0)
    }
}

/// Time scope for classes to be off.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TimeScope {
    /// All day to skip.
    ///
    /// Optionally contains the last day to skip.
    AllDay(Option<NaiveDate>),

    /// A period number to skip.
    Period(u8),
}

/// Describes a notice in series-department space.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Notice {
    /// Notice for an class suspension :D
    #[serde(rename_all = "camelCase")]
    ClassOff {
        /// The effective date of this notice.
        date: NaiveDate,

        /// How long the class would be off.
        time: TimeScope,

        #[serde(default, skip_serializing_if = "WhoScope::is_default")]
        /// For whom class would be off.
        for_whom: WhoScope,

        /// Would the day be skipped from calendar?
        day_off: bool,
    },
    /// Notice for an extra class :|
    #[serde(rename_all = "camelCase")]
    ExtraClass {
        /// The effective date of this notice.
        date: NaiveDate,

        /// Clock time for the class.
        time: DateTime<Local>,

        /// Who should attend the class.
        for_whom: WhoScope,
    },
    /// Notice for a class test :(
    #[allow(missing_docs)]
    #[serde(rename_all = "camelCase")]
    ClassTest {
        day: Day,
        cycle: u8,
        period: u8,
        course: String,
        teacher: String,

        /// Should contain syllabus.
        extra_info: String,
    },
    /// Notice for an exam :`(
    #[allow(missing_docs)]
    #[serde(rename_all = "camelCase")]
    Exam {
        /// The effective date of this notice.
        date: NaiveDate,

        course: String,

        /// Should contain syllabus.
        extra_info: String,
    },
    /// Other generic kind of notice.
    #[serde(rename_all = "camelCase")]
    Others {
        /// The effective date of this notice.
        date: NaiveDate,

        /// The message of this notice.
        message: String,
    },
}

/// Describes a holiday duration
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum HolidaySpan {
    /// One single holiday.
    #[serde(rename_all = "camelCase")]
    SingleDay {
        /// The day, the day.....
        on: NaiveDate,
    },

    /// More than one day of vacation.
    #[serde(rename_all = "camelCase")]
    MultiDays {
        /// Start date of this holiady.
        from: NaiveDate,

        /// End date of this holiady.
        to: NaiveDate,
    },
}

impl HolidaySpan {
    /// Get the starting day of this span.
    pub fn start(self) -> NaiveDate {
        match self {
            HolidaySpan::SingleDay { on } => on,
            HolidaySpan::MultiDays { from, .. } => from,
        }
    }

    /// Get the ending day of this span.
    pub fn end(self) -> NaiveDate {
        match self {
            HolidaySpan::SingleDay { on } => on,
            HolidaySpan::MultiDays { to, .. } => to,
        }
    }

    /// Check if this span contains the given date.
    pub fn contains(self, needle: NaiveDate) -> bool {
        match self {
            HolidaySpan::SingleDay { on } => needle == on,
            HolidaySpan::MultiDays { from, to } => needle >= from && needle <= to,
        }
    }
}

/// Describes an official holiday in RUET.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Holiday {
    /// Reason for this holiady.
    pub r#for: String,

    /// Start date of this holiady.
    #[serde(flatten)]
    pub span: HolidaySpan,
}

/// Describes a possible date-to-day relation.
#[derive(Debug, Clone)]
pub enum DateDayMapping {
    /// A regular class day.
    Day(Day),

    /// A weekend day.
    Weekend,

    /// An official holiday.
    Holiday(Holiday),

    /// An off day.
    OffDay(Notice),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml as yaml;
    use speculate::speculate;

    speculate! {
        it "should print a routine" {
            let routine: ClassRoutine = [
                (Day::A, vec![
                    ClassInRoutine {
                        course: "EEE2104".to_owned(),
                        teacher: "MFH".to_owned(),
                        period: 1,
                        contact_hours: 3,
                        class_room: "EEE 201".to_owned(),
                        frequency: ClassFrequency::EveryCycleWith(Thirty(2)),
                        comment: "".to_owned()
                    }
                ])
            ].iter().cloned().collect();

            let string = r#"
A:
  - course: EEE 2105
    teacher: SCM
    period: 4
    classRoom: EEE 201
  - course: Math 2101
    teacher: MSA
    period: 5
    classRoom: EEE 201
  - course: ME 2101
    teacher: RIS
    period: 6
    classRoom: EEE 201
                "#;

            println!("{:#?}", routine);
            println!("{}", yaml::to_string(&routine).unwrap());
            println!("{}", string);

            println!("{:#?}", yaml::from_str::<ClassRoutine>(string).unwrap());

            //println!("Result : {:#?}", result);
            //Current state of agenda :
            //println!("{}", json::to_string_pretty(&agenda).unwrap());
        }

        it "should be a holiday" {
            let holiady = Holiday {
                r#for: "What!!".to_string(),
                span: HolidaySpan::MultiDays {
                    from: NaiveDate::from_ymd(2020, 1, 1),
                    to: NaiveDate::from_ymd(2020, 1, 1)
                }
            };

            println!("{:#?}", holiady);
            println!("{}", yaml::to_string(&holiady).unwrap());
        }
    }
}
