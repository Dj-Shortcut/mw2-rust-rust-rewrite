use crate::gathering::height_at;
use crate::{Inventory, Item, Terrain};
use serde::{Deserialize, Deserializer, Serialize};

pub const TRADER_REACH: f32 = 100.;
pub const REQUEST_SECONDS: f32 = 600.;
const POSITION: [f32; 2] = [-500., -500.];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TradeOffer {
    pub price: (Item, u32),
    pub goods: (Item, u32),
}

pub const TRADE_OFFERS: [TradeOffer; 7] = [
    TradeOffer {
        price: (Item::Fish, 3),
        goods: (Item::Bandage, 2),
    },
    TradeOffer {
        price: (Item::CookedFish, 4),
        goods: (Item::Ammo, 30),
    },
    TradeOffer {
        price: (Item::CookedFish, 5),
        goods: (Item::Syringe, 1),
    },
    TradeOffer {
        price: (Item::Food, 10),
        goods: (Item::AntiRadPills, 1),
    },
    TradeOffer {
        price: (Item::Fish, 5),
        goods: (Item::Raincoat, 1),
    },
    TradeOffer {
        price: (Item::CookedFish, 3),
        goods: (Item::SignalFlare, 1),
    },
    TradeOffer {
        price: (Item::Honey, 3),
        goods: (Item::Barometer, 1),
    },
];

pub const TRADE_REQUESTS: [TradeOffer; 5] = [
    TradeOffer {
        price: (Item::Fish, 4),
        goods: (Item::Syringe, 1),
    },
    TradeOffer {
        price: (Item::Honey, 2),
        goods: (Item::Bandage, 4),
    },
    TradeOffer {
        price: (Item::CookedFish, 3),
        goods: (Item::AntiRadPills, 2),
    },
    TradeOffer {
        price: (Item::BerryTea, 2),
        goods: (Item::Ammo, 60),
    },
    TradeOffer {
        price: (Item::FishStew, 1),
        goods: (Item::SignalFlare, 1),
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SavedRequest {
    index: u32,
    remaining: f32,
    fulfilled: bool,
}

impl Default for SavedRequest {
    fn default() -> Self {
        Self {
            index: 0,
            remaining: REQUEST_SECONDS,
            fulfilled: false,
        }
    }
}

impl<'de> Deserialize<'de> for SavedRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            index: u32,
            remaining: f32,
            fulfilled: bool,
        }
        let saved = Saved::deserialize(deserializer)?;
        if saved.index as usize >= TRADE_REQUESTS.len() {
            return Err(serde::de::Error::custom(
                "Saved trader request does not exist",
            ));
        }
        if !saved.remaining.is_finite()
            || saved.remaining <= 0.
            || saved.remaining > REQUEST_SECONDS
        {
            return Err(serde::de::Error::custom(
                "Saved trader request timer is out of range",
            ));
        }
        Ok(Self {
            index: saved.index,
            remaining: saved.remaining,
            fulfilled: saved.fulfilled,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TradingPost {
    position: [f32; 3],
    request: SavedRequest,
}

impl TradingPost {
    pub(crate) fn new(terrain: &Terrain) -> Result<Self, String> {
        let z = height_at(terrain, POSITION).ok_or("Trading post lies outside terrain mesh")?;
        Ok(Self {
            position: [POSITION[0], POSITION[1], z],
            request: SavedRequest::default(),
        })
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn in_reach(&self, origin: [f32; 3]) -> bool {
        (0..3)
            .map(|k| (self.position[k] - origin[k]).powi(2))
            .sum::<f32>()
            .sqrt()
            <= TRADER_REACH
    }

    pub fn trader_request(&self) -> TradeOffer {
        TRADE_REQUESTS[self.request.index as usize]
    }

    pub fn request_fulfilled(&self) -> bool {
        self.request.fulfilled
    }

    pub fn request_remaining(&self) -> f32 {
        self.request.remaining
    }

    pub(crate) fn advance(&mut self, dt_seconds: f32) -> Result<(), String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Trader request time step must be finite and non-negative".into());
        }
        let request = &mut self.request;
        request.remaining -= dt_seconds;
        if request.remaining <= 0. {
            request.index = (request.index + 1) % TRADE_REQUESTS.len() as u32;
            request.remaining = REQUEST_SECONDS;
            request.fulfilled = false;
        }
        Ok(())
    }

    pub(crate) fn fulfill(&mut self, inventory: &mut Inventory) -> Result<TradeOffer, String> {
        if self.request.fulfilled {
            return Err("The trader's request is already filled".into());
        }
        let request = self.trader_request();
        exchange(inventory, request)?;
        self.request.fulfilled = true;
        Ok(request)
    }

    pub(crate) fn saved(&self) -> SavedRequest {
        self.request
    }

    pub(crate) fn restore(&mut self, saved: SavedRequest) {
        self.request = saved;
    }

    pub(crate) fn trade(inventory: &mut Inventory, offer: usize) -> Result<TradeOffer, String> {
        let offer = *TRADE_OFFERS.get(offer).ok_or("No such trade offer")?;
        exchange(inventory, offer)?;
        Ok(offer)
    }
}

fn exchange(inventory: &mut Inventory, offer: TradeOffer) -> Result<(), String> {
    let (paid, price) = offer.price;
    if inventory.count(paid) < price {
        return Err(format!("You need {price} {}", paid.name()));
    }
    let mut traded = inventory.clone();
    traded.take(paid, price)?;
    let (item, quantity) = offer.goods;
    traded
        .add(item, quantity)
        .map_err(|_| "Not enough inventory space for the trade")?;
    *inventory = traded;
    Ok(())
}
