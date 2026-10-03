use crate::gathering::height_at;
use crate::{Inventory, Item, Terrain};

pub const TRADER_REACH: f32 = 100.;
const POSITION: [f32; 2] = [-500., -500.];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TradeOffer {
    pub price: (Item, u32),
    pub goods: (Item, u32),
}

pub const TRADE_OFFERS: [TradeOffer; 5] = [
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
];

#[derive(Clone, Debug, PartialEq)]
pub struct TradingPost {
    position: [f32; 3],
}

impl TradingPost {
    pub(crate) fn new(terrain: &Terrain) -> Result<Self, String> {
        let z = height_at(terrain, POSITION).ok_or("Trading post lies outside terrain mesh")?;
        Ok(Self {
            position: [POSITION[0], POSITION[1], z],
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

    pub(crate) fn trade(inventory: &mut Inventory, offer: usize) -> Result<TradeOffer, String> {
        let offer = *TRADE_OFFERS.get(offer).ok_or("No such trade offer")?;
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
        Ok(offer)
    }
}
