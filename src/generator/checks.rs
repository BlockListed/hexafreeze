use crate::{
    constants,
    error::{HexaFreezeError, HexaFreezeResult},
};
use chrono::prelude::*;
use chrono::Duration;

pub const fn check_node_id(id: i64) -> HexaFreezeResult<()> {
    if id > crate::constants::MAX_NODE_ID {
        return Err(HexaFreezeError::NodeIdTooLarge);
    }

    Ok(())
}

pub fn check_epoch(epoch: DateTime<Utc>) -> HexaFreezeResult<()> {
    let now = super::util::now();

    if now - epoch < Duration::seconds(0) {
        return Err(HexaFreezeError::EpochInTheFuture);
    }

    if now - epoch > Duration::from_std(constants::MAX_TIMESTAMP).unwrap() {
        return Err(HexaFreezeError::EpochTooFarInThePast);
    }

    Ok(())
}
