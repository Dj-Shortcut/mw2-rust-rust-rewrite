use crate::Terrain;
use crate::gathering::height_at;
use serde::{Deserialize, Deserializer, Serialize};

pub const GROW_SECONDS: f32 = 600.;
pub const PLOT_REACH: f32 = 100.;
pub const HARVEST_FOOD: u32 = 5;
pub const HARVEST_SEEDS: u32 = 2;
const LAYOUT: [[f32; 2]; 3] = [[180., 160.], [260., 160.], [340., 160.]];

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum PlotState {
    Empty,
    Growing { remaining: f32 },
    Ripe,
}

impl<'de> Deserialize<'de> for PlotState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Saved {
            Empty,
            Growing { remaining: f32 },
            Ripe,
        }
        Ok(match Saved::deserialize(deserializer)? {
            Saved::Empty => Self::Empty,
            Saved::Ripe => Self::Ripe,
            Saved::Growing { remaining } => {
                if !remaining.is_finite() || remaining <= 0. || remaining > GROW_SECONDS {
                    return Err(serde::de::Error::custom("Growing time is out of range"));
                }
                Self::Growing { remaining }
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plot {
    id: u32,
    position: [f32; 3],
    state: PlotState,
}

impl Plot {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn state(&self) -> PlotState {
        self.state
    }

    fn distance(&self, origin: [f32; 3]) -> f32 {
        (0..3)
            .map(|k| (self.position[k] - origin[k]).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

/// Saved plot states in layout order; empty in saves from before farming.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SavedGarden {
    states: Vec<PlotState>,
}

impl<'de> Deserialize<'de> for SavedGarden {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let states = Vec::<PlotState>::deserialize(deserializer)?;
        if !states.is_empty() && states.len() != LAYOUT.len() {
            return Err(serde::de::Error::custom(
                "Saved garden plots do not match the world",
            ));
        }
        Ok(Self { states })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Garden {
    plots: Vec<Plot>,
}

impl Garden {
    pub(crate) fn new(terrain: &Terrain) -> Result<Self, String> {
        let plots = LAYOUT
            .iter()
            .enumerate()
            .map(|(index, &xy)| {
                let z = height_at(terrain, xy).ok_or("Garden plot lies outside terrain mesh")?;
                Ok(Plot {
                    id: index as u32 + 1,
                    position: [xy[0], xy[1], z],
                    state: PlotState::Empty,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(Self { plots })
    }

    pub fn plots(&self) -> &[Plot] {
        &self.plots
    }

    pub fn in_reach(&self, origin: [f32; 3]) -> Option<&Plot> {
        self.plots
            .iter()
            .filter(|p| p.distance(origin) <= PLOT_REACH)
            .min_by(|a, b| a.distance(origin).total_cmp(&b.distance(origin)))
    }

    fn plot_mut(&mut self, id: u32) -> Result<&mut Plot, String> {
        self.plots
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| "No such garden plot".into())
    }

    pub(crate) fn plant(&mut self, id: u32) -> Result<(), String> {
        let plot = self.plot_mut(id)?;
        match plot.state {
            PlotState::Empty => {
                plot.state = PlotState::Growing {
                    remaining: GROW_SECONDS,
                };
                Ok(())
            }
            PlotState::Growing { .. } => Err("Something is already growing here".into()),
            PlotState::Ripe => Err("Harvest the ripe berries first".into()),
        }
    }

    pub(crate) fn harvest(&mut self, id: u32) -> Result<(), String> {
        let plot = self.plot_mut(id)?;
        match plot.state {
            PlotState::Ripe => {
                plot.state = PlotState::Empty;
                Ok(())
            }
            PlotState::Growing { .. } => Err("The berries are not ripe yet".into()),
            PlotState::Empty => Err("Nothing is planted here".into()),
        }
    }

    pub(crate) fn advance(&mut self, dt_seconds: f32) -> Result<bool, String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Invalid growing time step".into());
        }
        let mut ripened = false;
        for plot in &mut self.plots {
            if let PlotState::Growing { remaining } = plot.state {
                let left = remaining - dt_seconds;
                plot.state = if left <= 0. {
                    ripened = true;
                    PlotState::Ripe
                } else {
                    PlotState::Growing { remaining: left }
                };
            }
        }
        Ok(ripened)
    }

    pub(crate) fn saved(&self) -> SavedGarden {
        SavedGarden {
            states: self.plots.iter().map(|p| p.state).collect(),
        }
    }

    pub(crate) fn restore(&mut self, saved: &SavedGarden) {
        for (index, plot) in self.plots.iter_mut().enumerate() {
            plot.state = saved.states.get(index).copied().unwrap_or(PlotState::Empty);
        }
    }
}
