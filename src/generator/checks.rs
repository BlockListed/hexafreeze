use super::nano::Time;
use crate::{
    constants,
    error::{HexaFreezeError, HexaFreezeResult},
};
use uom::si::time::second;

pub const fn check_node_id(id: i64) -> HexaFreezeResult<()> {
    if id > crate::constants::MAX_NODE_ID {
        return Err(HexaFreezeError::NodeIdTooLarge);
    }

    Ok(())
}

pub fn check_epoch(epoch: Time) -> HexaFreezeResult<()> {
    let now = super::util::now();

    if now - epoch < Time::new::<second>(0) {
        return Err(HexaFreezeError::EpochInTheFuture);
    }

    if now - epoch > *constants::MAX_TIMESTAMP {
        return Err(HexaFreezeError::EpochTooFarInThePast);
    }

    Ok(())
}
