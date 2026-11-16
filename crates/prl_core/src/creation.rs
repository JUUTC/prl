//! Headless creation drafts and accepted starting-budget constraints (Q-033).
//!
//! This is not a playable character, save format, random generator or skill
//! catalogue. Callers supply class defaults, rolled values and allowed skill
//! IDs; future integration must validate gear, race and class availability.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use prl_content::ContentId;

/// Independent creation-section mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreationMode {
    /// Supplied defaults for the selected class.
    Default,
    /// Player-editable draft, retained across class changes.
    Customize,
    /// Caller-supplied roll; edits require switching to another mode.
    Random,
}

/// Six pre-racial scores in STR, DEX, CON, INT, WIS, CHA order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartingStats {
    /// Scores before racial modifiers.
    pub scores: [u8; 6],
}

/// Starting skill values keyed by content ID, in tenths of a skill point.
pub type StartingSkills = BTreeMap<ContentId, u16>;

/// Why a creation draft operation or allocation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreationError {
    /// A score was outside the mode's allowed range.
    StatOutOfRange {
        /// Zero-based index in [`StartingStats::scores`].
        index: usize,
        /// Supplied score.
        score: u8,
        /// Maximum for the selected mode (minimum is always 8).
        maximum: u8,
    },
    /// Stat point-buy total does not fit the selected mode.
    StatBudget {
        /// Actual cost.
        spent: u16,
        /// Inclusive minimum.
        minimum: u16,
        /// Inclusive maximum.
        maximum: u16,
    },
    /// Skill does not exist in the caller-supplied creation catalogue.
    UnknownSkill(ContentId),
    /// One starting skill exceeds 50.0.
    SkillTooHigh {
        /// Invalid skill.
        skill: ContentId,
        /// Supplied value in tenths.
        tenths: u16,
    },
    /// Skill point total differs from the selected mode's budget.
    SkillBudget {
        /// Actual total in tenths.
        spent_tenths: u64,
        /// Required total in tenths.
        required_tenths: u16,
    },
    /// The operation requires a different section mode.
    WrongMode {
        /// Expected section mode.
        required: CreationMode,
        /// Current section mode.
        actual: CreationMode,
    },
}

impl fmt::Display for CreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StatOutOfRange {
                index,
                score,
                maximum,
            } => {
                write!(
                    f,
                    "stat {index} score {score} must be between 8 and {maximum}"
                )
            }
            Self::StatBudget {
                spent,
                minimum,
                maximum,
            } => {
                write!(
                    f,
                    "stat cost {spent} must be between {minimum} and {maximum}"
                )
            }
            Self::UnknownSkill(skill) => write!(f, "unknown starting skill {skill}"),
            Self::SkillTooHigh { skill, tenths } => {
                write!(
                    f,
                    "starting skill {skill} is {tenths} tenths; maximum is 500"
                )
            }
            Self::SkillBudget {
                spent_tenths,
                required_tenths,
            } => {
                write!(
                    f,
                    "starting skills total {spent_tenths} tenths; expected {required_tenths}"
                )
            }
            Self::WrongMode { required, actual } => {
                write!(f, "operation requires {required:?} mode, not {actual:?}")
            }
        }
    }
}

impl std::error::Error for CreationError {}

/// Calculates pre-racial point-buy cost.
///
/// # Errors
/// Rejects scores outside 8..=15 for Default/Customize or 8..=18 for Random.
pub fn stat_cost(stats: StartingStats, mode: CreationMode) -> Result<u16, CreationError> {
    const COSTS: [u16; 11] = [0, 1, 2, 3, 4, 5, 7, 9, 12, 15, 18];
    let maximum = if mode == CreationMode::Random { 18 } else { 15 };
    stats
        .scores
        .into_iter()
        .enumerate()
        .try_fold(0, |sum, (index, score)| {
            if !(8..=maximum).contains(&score) {
                return Err(CreationError::StatOutOfRange {
                    index,
                    score,
                    maximum,
                });
            }
            // The range check above bounds the index independently of caller input.
            Ok(sum + COSTS[usize::from(score - 8)])
        })
}

/// Validates the complete pre-racial stat allocation for its mode.
///
/// # Errors
/// Reports an out-of-range score or a total outside the accepted budget.
pub fn validate_stats(stats: StartingStats, mode: CreationMode) -> Result<(), CreationError> {
    let spent = stat_cost(stats, mode)?;
    let (minimum, maximum) = if mode == CreationMode::Random {
        (24, 30)
    } else {
        (27, 27)
    };
    if !(minimum..=maximum).contains(&spent) {
        return Err(CreationError::StatBudget {
            spent,
            minimum,
            maximum,
        });
    }
    Ok(())
}

fn validate_skill_entries(
    skills: &StartingSkills,
    catalogue: &BTreeSet<ContentId>,
) -> Result<u64, CreationError> {
    let mut total = 0_u64;
    for (skill, &tenths) in skills {
        if !catalogue.contains(skill) {
            return Err(CreationError::UnknownSkill(skill.clone()));
        }
        if tenths > 500 {
            return Err(CreationError::SkillTooHigh {
                skill: skill.clone(),
                tenths,
            });
        }
        total += u64::from(tenths);
    }
    Ok(total)
}

/// Validates complete starting skills against the supplied catalogue and budget.
///
/// Values are fixed-point tenths; absent catalogue skills have zero points.
/// The 50.0 cap is a creation restriction, not a progression skill cap.
///
/// # Errors
/// Reports unknown IDs, an individual value above 50.0 or a wrong total.
pub fn validate_skills(
    skills: &StartingSkills,
    catalogue: &BTreeSet<ContentId>,
    mode: CreationMode,
) -> Result<(), CreationError> {
    let spent_tenths = validate_skill_entries(skills, catalogue)?;
    let required_tenths = if mode == CreationMode::Random {
        1_800
    } else {
        1_500
    };
    if spent_tenths != u64::from(required_tenths) {
        return Err(CreationError::SkillBudget {
            spent_tenths,
            required_tenths,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Section<T> {
    mode: CreationMode,
    value: T,
}

impl<T> Section<T> {
    fn default(value: T) -> Self {
        Self {
            mode: CreationMode::Default,
            value,
        }
    }

    fn require(&self, mode: CreationMode) -> Result<(), CreationError> {
        if self.mode == mode {
            Ok(())
        } else {
            Err(CreationError::WrongMode {
                required: mode,
                actual: self.mode,
            })
        }
    }

    fn customize(&mut self) {
        self.mode = CreationMode::Customize;
    }

    fn reset(&mut self, value: T) {
        *self = Self::default(value);
    }

    fn roll(&mut self, value: T) {
        *self = Self {
            mode: CreationMode::Random,
            value,
        };
    }

    fn class_default(&mut self, value: T) {
        if self.mode == CreationMode::Default {
            self.value = value;
        }
    }
}

/// Immutable view of one creation section.
#[derive(Debug, Clone, Copy)]
pub struct CreationSection<'a, T> {
    /// Current mode.
    pub mode: CreationMode,
    /// Current draft or caller-provided value.
    pub value: &'a T,
}

/// Headless creation draft with independently managed sections.
///
/// `G` is an opaque caller-owned gear draft. No gear legality, budget, identity
/// or random-weight rule is implied by storing it here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreationDraft<G> {
    class: ContentId,
    skills_catalogue: BTreeSet<ContentId>,
    stats: Section<StartingStats>,
    skills: Section<StartingSkills>,
    gear: Section<G>,
}

impl<G> CreationDraft<G> {
    /// Creates a draft from supplied class defaults and a skill catalogue.
    ///
    /// # Errors
    /// Rejects invalid default stat or skill allocations.
    pub fn new(
        class: ContentId,
        skills_catalogue: BTreeSet<ContentId>,
        stats: StartingStats,
        skills: StartingSkills,
        gear: G,
    ) -> Result<Self, CreationError> {
        validate_stats(stats, CreationMode::Default)?;
        validate_skills(&skills, &skills_catalogue, CreationMode::Default)?;
        Ok(Self {
            class,
            skills_catalogue,
            stats: Section::default(stats),
            skills: Section::default(skills),
            gear: Section::default(gear),
        })
    }

    /// Selected class ID; availability is owned by the caller.
    #[must_use]
    pub fn class(&self) -> &ContentId {
        &self.class
    }

    /// Current pre-racial stat section.
    #[must_use]
    pub fn stats(&self) -> CreationSection<'_, StartingStats> {
        CreationSection {
            mode: self.stats.mode,
            value: &self.stats.value,
        }
    }

    /// Current starting skill section.
    #[must_use]
    pub fn skills(&self) -> CreationSection<'_, StartingSkills> {
        CreationSection {
            mode: self.skills.mode,
            value: &self.skills.value,
        }
    }

    /// Current opaque gear section.
    #[must_use]
    pub fn gear(&self) -> CreationSection<'_, G> {
        CreationSection {
            mode: self.gear.mode,
            value: &self.gear.value,
        }
    }

    /// Changes class and updates only Default sections, atomically.
    ///
    /// Customized and random values remain untouched; a caller may explicitly
    /// reroll after a class change. The supplied gear is not validated here.
    ///
    /// # Errors
    /// Rejects invalid new class stat or skill defaults without modifying the draft.
    pub fn change_class(
        &mut self,
        class: ContentId,
        stats: StartingStats,
        skills: StartingSkills,
        gear: G,
    ) -> Result<(), CreationError> {
        validate_stats(stats, CreationMode::Default)?;
        validate_skills(&skills, &self.skills_catalogue, CreationMode::Default)?;
        self.class = class;
        self.stats.class_default(stats);
        self.skills.class_default(skills);
        self.gear.class_default(gear);
        Ok(())
    }

    /// Enables stat editing without changing its current value.
    pub fn customize_stats(&mut self) {
        self.stats.customize();
    }
    /// Enables skill editing without changing its current value.
    pub fn customize_skills(&mut self) {
        self.skills.customize();
    }
    /// Enables gear editing without changing its current value.
    pub fn customize_gear(&mut self) {
        self.gear.customize();
    }

    /// Replaces editable stats; an incomplete budget is allowed while editing.
    ///
    /// # Errors
    /// Rejects a non-custom section or an out-of-range pre-racial score.
    pub fn edit_stats(&mut self, stats: StartingStats) -> Result<(), CreationError> {
        self.stats.require(CreationMode::Customize)?;
        stat_cost(stats, CreationMode::Customize)?;
        self.stats.value = stats;
        Ok(())
    }

    /// Replaces editable skills; an incomplete budget is allowed while editing.
    ///
    /// # Errors
    /// Rejects a non-custom section, unknown skill or value above 50.0.
    pub fn edit_skills(&mut self, skills: StartingSkills) -> Result<(), CreationError> {
        self.skills.require(CreationMode::Customize)?;
        validate_skill_entries(&skills, &self.skills_catalogue)?;
        self.skills.value = skills;
        Ok(())
    }

    /// Replaces the opaque gear draft while in Customize mode.
    ///
    /// # Errors
    /// Rejects editing in Default or Random mode.
    pub fn edit_gear(&mut self, gear: G) -> Result<(), CreationError> {
        self.gear.require(CreationMode::Customize)?;
        self.gear.value = gear;
        Ok(())
    }

    /// Applies a caller-supplied stat roll; repeating this freely rerolls it.
    ///
    /// # Errors
    /// Rejects a roll outside the Random stat bounds or budget.
    pub fn roll_stats(&mut self, stats: StartingStats) -> Result<(), CreationError> {
        validate_stats(stats, CreationMode::Random)?;
        self.stats.roll(stats);
        Ok(())
    }

    /// Applies a caller-supplied skill roll; repeating this freely rerolls it.
    ///
    /// # Errors
    /// Rejects an unknown skill, value above 50.0, or non-180-point total.
    pub fn roll_skills(&mut self, skills: StartingSkills) -> Result<(), CreationError> {
        validate_skills(&skills, &self.skills_catalogue, CreationMode::Random)?;
        self.skills.roll(skills);
        Ok(())
    }

    /// Applies a caller-supplied gear roll; gear validity is not checked here.
    pub fn roll_gear(&mut self, gear: G) {
        self.gear.roll(gear);
    }

    /// Explicitly resets stats to supplied class defaults.
    ///
    /// # Errors
    /// Rejects invalid default stats without modifying the section.
    pub fn reset_stats(&mut self, stats: StartingStats) -> Result<(), CreationError> {
        validate_stats(stats, CreationMode::Default)?;
        self.stats.reset(stats);
        Ok(())
    }

    /// Explicitly resets skills to supplied class defaults.
    ///
    /// # Errors
    /// Rejects invalid default skills without modifying the section.
    pub fn reset_skills(&mut self, skills: StartingSkills) -> Result<(), CreationError> {
        validate_skills(&skills, &self.skills_catalogue, CreationMode::Default)?;
        self.skills.reset(skills);
        Ok(())
    }

    /// Explicitly resets the opaque gear section to supplied class defaults.
    pub fn reset_gear(&mut self, gear: G) {
        self.gear.reset(gear);
    }

    /// Validates only completed stat and skill allocations, not the entire character.
    ///
    /// # Errors
    /// Reports an invalid current stat or skill allocation.
    pub fn validate_allocations(&self) -> Result<(), CreationError> {
        validate_stats(self.stats.value, self.stats.mode)?;
        validate_skills(&self.skills.value, &self.skills_catalogue, self.skills.mode)
    }
}

#[cfg(test)]
mod tests;
