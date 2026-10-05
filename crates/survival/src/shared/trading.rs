use super::{
    ActorHandle, SHARED_TRADE_REACH, SHARED_TRADE_TICKS, SHARED_TRADE_WOOD, SharedEffect,
    SharedSession, SharedTradeOffer, SharedTradeOutcome, SharedTradeOutcomeKind, SharedTradeParty,
    persistence,
};
use crate::{Inventory, Item};
use rust_building::{BuildingWorld, Resources};
use sim::Tick;

#[derive(Clone, Copy)]
pub(super) struct LiveTrade {
    offer: SharedTradeOffer,
    seller: ActorHandle,
    buyer: ActorHandle,
}

impl LiveTrade {
    fn involves(self, actor: ActorHandle) -> bool {
        self.seller == actor || self.buyer == actor
    }
}

pub(super) fn settle_trade(
    seller: &Inventory,
    buyer: &Inventory,
    buildings: &BuildingWorld,
    seller_owner: u32,
    buyer_owner: u32,
) -> Result<(Inventory, Inventory, BuildingWorld), String> {
    if seller_owner == buyer_owner {
        return Err("Trading needs two distinct owners".into());
    }
    let mut seller_inventory = seller.clone();
    let mut buyer_inventory = buyer.clone();
    let mut ledger = buildings.clone();
    seller_inventory
        .take(Item::Bandage, 1)
        .map_err(|_| "Seller no longer has a Bandage")?;
    buyer_inventory.add(Item::Bandage, 1)?;
    let payment = Resources {
        wood: SHARED_TRADE_WOOD,
        ..Default::default()
    };
    ledger
        .consume(buyer_owner, payment)
        .map_err(|_| format!("Buyer needs {SHARED_TRADE_WOOD} Wood"))?;
    if ledger
        .inventory(seller_owner)
        .wood
        .checked_add(SHARED_TRADE_WOOD)
        .is_none_or(|total| total > persistence::MAX_RESOURCE_BALANCE)
    {
        return Err("Seller resource storage is full".into());
    }
    ledger
        .grant(seller_owner, payment)
        .map_err(|error| error.to_string())?;
    Ok((seller_inventory, buyer_inventory, ledger))
}

impl SharedSession {
    pub(super) fn trade_offer_for(&self, actor: ActorHandle) -> Option<SharedTradeOffer> {
        self.trade
            .filter(|trade| trade.involves(actor))
            .map(|trade| trade.offer)
    }

    pub(super) fn offer_bandage(&mut self, seller: ActorHandle) -> Result<SharedEffect, String> {
        if self.trade.is_some() {
            return Err("A trade offer is already active".into());
        }
        if self.actor(seller)?.inventory.count(Item::Bandage) == 0 {
            return Err("You need a Bandage to offer".into());
        }
        let mut peers = self.actors.values().filter(|actor| {
            actor.handle != seller
                && !actor.initializing
                && self
                    .world
                    .player(actor.handle.client)
                    .is_some_and(|player| player.health > 0)
        });
        let buyer = peers
            .next()
            .map(|actor| actor.handle)
            .ok_or("No eligible player to trade with")?;
        if peers.next().is_some() {
            return Err("Trade target is ambiguous".into());
        }
        self.trade_pair_in_reach(seller, buyer)?;
        let expires_at = self
            .tick
            .checked_add(SHARED_TRADE_TICKS)
            .ok_or("Trade clock exhausted")?;
        let next_id = self
            .next_trade_id
            .checked_add(1)
            .ok_or("Trade offer IDs exhausted")?;
        let offer = SharedTradeOffer {
            id: self.next_trade_id,
            seller: party(seller),
            buyer: party(buyer),
            created_at: Tick(self.tick),
            expires_at: Tick(expires_at),
        };
        self.trade = Some(LiveTrade {
            offer,
            seller,
            buyer,
        });
        self.next_trade_id = next_id;
        Ok(SharedEffect::TradeOffered { offer_id: offer.id })
    }

    pub(super) fn accept_trade(
        &mut self,
        actor: ActorHandle,
        offer_id: u64,
    ) -> Result<SharedEffect, String> {
        let trade = self.current_trade(actor, offer_id)?;
        if trade.buyer != actor {
            return Err("Only the addressed buyer can accept this offer".into());
        }
        self.trade_pair_in_reach(trade.seller, trade.buyer)?;
        let (seller_inventory, buyer_inventory, buildings) = settle_trade(
            &self.actor(trade.seller)?.inventory,
            &self.actor(trade.buyer)?.inventory,
            self.world.buildings(),
            trade.seller.owner,
            trade.buyer.owner,
        )?;
        self.actors
            .get_mut(&trade.seller.client)
            .expect("validated seller remains active")
            .inventory = seller_inventory;
        self.actors
            .get_mut(&trade.buyer.client)
            .expect("validated buyer remains active")
            .inventory = buyer_inventory;
        *self.world.buildings_mut() = buildings;
        self.finish_trade(SharedTradeOutcomeKind::Accepted);
        Ok(SharedEffect::TradeAccepted { offer_id })
    }

    pub(super) fn close_trade(
        &mut self,
        actor: ActorHandle,
        offer_id: u64,
    ) -> Result<SharedEffect, String> {
        let trade = self.current_trade(actor, offer_id)?;
        let kind = if trade.seller == actor {
            SharedTradeOutcomeKind::Cancelled
        } else {
            SharedTradeOutcomeKind::Declined
        };
        self.finish_trade(kind);
        Ok(SharedEffect::TradeClosed { offer_id })
    }

    pub(super) fn invalidate_departing_trade(&mut self, actor: ActorHandle) {
        if self.trade.is_some_and(|trade| trade.involves(actor)) {
            self.finish_trade(SharedTradeOutcomeKind::Unavailable);
        }
    }

    pub(super) fn advance_trade(&mut self) {
        let Some(trade) = self.trade else {
            return;
        };
        let unavailable = [trade.seller, trade.buyer].into_iter().any(|handle| {
            self.actor(handle).is_err()
                || !self
                    .world
                    .player(handle.client)
                    .is_some_and(|player| player.health > 0)
        });
        if unavailable {
            self.finish_trade(SharedTradeOutcomeKind::Unavailable);
        } else if self.tick >= trade.offer.expires_at.0 {
            self.finish_trade(SharedTradeOutcomeKind::Expired);
        }
    }

    fn current_trade(&self, actor: ActorHandle, offer_id: u64) -> Result<LiveTrade, String> {
        self.trade
            .filter(|trade| trade.offer.id == offer_id && trade.involves(actor))
            .ok_or_else(|| "Trade offer is no longer available".into())
    }

    fn trade_pair_in_reach(&self, seller: ActorHandle, buyer: ActorHandle) -> Result<(), String> {
        if self.actor(seller)?.initializing || self.actor(buyer)?.initializing {
            return Err("Trading players are initializing".into());
        }
        let players = [seller, buyer].map(|actor| {
            self.world
                .player(actor.client)
                .filter(|player| player.health > 0)
                .ok_or("Trading player is not alive")
        });
        let [seller_player, buyer_player] = players;
        let seller_player = seller_player?;
        let buyer_player = buyer_player?;
        let distance_squared: f32 = (0..3)
            .map(|axis| (seller_player.origin[axis] - buyer_player.origin[axis]).powi(2))
            .sum();
        if !distance_squared.is_finite() || distance_squared > SHARED_TRADE_REACH.powi(2) {
            return Err("Trading player is out of reach".into());
        }
        let eyes = [seller_player, buyer_player].map(|player| {
            let mut eye = player.origin;
            eye[2] += player.view_height_current;
            eye
        });
        if eyes.iter().flatten().any(|value| !value.is_finite()) {
            return Err("Trading sight line is invalid".into());
        }
        let sight = self
            .world
            .trace_world(eyes[0], eyes[1], [0.; 3], [0.; 3], 0x11);
        if !sight.fraction.is_finite()
            || sight.startsolid != 0
            || sight.allsolid != 0
            || sight.fraction < 1.
        {
            return Err("Trading player is behind an obstacle".into());
        }
        Ok(())
    }

    fn finish_trade(&mut self, kind: SharedTradeOutcomeKind) {
        let trade = self.trade.take().expect("a live trade was validated");
        let outcome = SharedTradeOutcome {
            offer_id: trade.offer.id,
            seller: trade.offer.seller,
            buyer: trade.offer.buyer,
            closed_at: Tick(self.tick),
            kind,
        };
        for handle in [trade.seller, trade.buyer] {
            if let Some(actor) = self
                .actors
                .get_mut(&handle.client)
                .filter(|actor| actor.handle == handle)
            {
                actor.trade_outcome = Some(outcome);
            }
        }
    }
}

fn party(actor: ActorHandle) -> SharedTradeParty {
    SharedTradeParty {
        client: actor.client,
        owner: actor.owner,
    }
}
