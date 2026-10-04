use crate::{Inventory, Recipe};
use rust_building::Resources;
use serde::{Deserialize, Deserializer, Serialize};

pub const MAX_CRAFT_JOBS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CraftJob {
    pub recipe: Recipe,
    pub remaining: f32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub cloth_reserved: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl CraftJob {
    pub fn resource_refund(self) -> Resources {
        if self.recipe == Recipe::Bandage && !self.cloth_reserved {
            Resources {
                wood: 20,
                ..Default::default()
            }
        } else {
            self.recipe.cost()
        }
    }

    pub fn cloth_refund(self) -> u32 {
        if self.recipe == Recipe::Bandage && self.cloth_reserved {
            4
        } else {
            0
        }
    }
}

impl<'de> Deserialize<'de> for CraftJob {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            recipe: Recipe,
            remaining: f32,
            #[serde(default)]
            cloth_reserved: bool,
        }
        let saved = Saved::deserialize(deserializer)?;
        if saved.cloth_reserved && saved.recipe != Recipe::Bandage {
            return Err(serde::de::Error::custom(
                "Only Bandage crafting jobs reserve Cloth",
            ));
        }
        Ok(Self {
            recipe: saved.recipe,
            remaining: saved.remaining,
            cloth_reserved: saved.cloth_reserved,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CraftQueue {
    jobs: Vec<CraftJob>,
}

impl CraftQueue {
    pub fn jobs(&self) -> &[CraftJob] {
        &self.jobs
    }

    pub(crate) fn push(&mut self, recipe: Recipe) -> Result<(), String> {
        if self.jobs.len() >= MAX_CRAFT_JOBS {
            return Err("Crafting queue is full".into());
        }
        self.jobs.push(CraftJob {
            recipe,
            remaining: recipe.craft_seconds(),
            cloth_reserved: recipe == Recipe::Bandage,
        });
        Ok(())
    }

    pub(crate) fn cancel(&mut self, index: usize) -> Result<CraftJob, String> {
        if index >= self.jobs.len() {
            return Err("No crafting job at that position".into());
        }
        Ok(self.jobs.remove(index))
    }

    pub(crate) fn clear(&mut self) -> (Resources, u32) {
        let mut refund = Resources::default();
        let mut cloth = 0;
        for job in self.jobs.drain(..) {
            refund.add(job.resource_refund());
            cloth += job.cloth_refund();
        }
        (refund, cloth)
    }

    pub(crate) fn advance(
        &mut self,
        dt_seconds: f32,
        inventory: &mut Inventory,
    ) -> Result<Option<Recipe>, String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Invalid crafting time step".into());
        }
        let Some(job) = self.jobs.first_mut() else {
            return Ok(None);
        };
        job.remaining = (job.remaining - dt_seconds).max(0.);
        if job.remaining > 0. {
            return Ok(None);
        }
        let (item, quantity) = job.recipe.output();
        if inventory.add(item, quantity).is_err() {
            return Ok(None);
        }
        Ok(Some(self.jobs.remove(0).recipe))
    }
}

impl<'de> Deserialize<'de> for CraftQueue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let jobs = Vec::<CraftJob>::deserialize(deserializer)?;
        if jobs.len() > MAX_CRAFT_JOBS {
            return Err(serde::de::Error::custom("Too many crafting jobs"));
        }
        for (index, job) in jobs.iter().enumerate() {
            let full = job.recipe.craft_seconds();
            let valid = if index == 0 {
                job.remaining.is_finite() && (0. ..=full).contains(&job.remaining)
            } else {
                job.remaining == full
            };
            if !valid {
                return Err(serde::de::Error::custom(
                    "Crafting job time is out of range",
                ));
            }
        }
        Ok(Self { jobs })
    }
}
