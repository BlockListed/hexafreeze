use crate::{
    constants,
    error::{HexaFreezeError, HexaFreezeResult},
    generator::nano::Nanosecond,
};

pub const fn check_node_id(id: i64) -> HexaFreezeResult<()> {
    if id > crate::constants::MAX_NODE_ID {
        return Err(HexaFreezeError::NodeIdTooLarge);
    }

    Ok(())
}

pub fn check_epoch(epoch: Nanosecond) -> HexaFreezeResult<()> {
    let now = super::util::now();

    if now - epoch < Nanosecond(0) {
        return Err(HexaFreezeError::EpochInTheFuture);
    }

    if now - epoch > constants::MAX_TIMESTAMP {
        return Err(HexaFreezeError::EpochTooFarInThePast);
    }

    Ok(())
}
