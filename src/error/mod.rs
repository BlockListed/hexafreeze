pub type HexaFreezeResult<T> = Result<T, HexaFreezeError>;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum HexaFreezeError {
    #[error("Epoch is too far in the future!")]
    // Epoch is too far in the future!
    EpochInTheFuture,
    #[error("Epoch is too far in the past!")]
    // Epoch is too far in the past!
    EpochTooFarInThePast,
    #[error("The node_id less than 0 or bigger than 1023")]
    /// The node_id less than 0 or bigger than 1023
    NodeIdTooLarge,
    #[error("Your clock jumped backwards in time!")]
    /// Your clock jumped backwards in time!
    ClockWentBackInTime,
    #[error("You've generated more than 9,223,372,036,854,775,807 ids!")]
    // ID generation surpassed 64 bit limit.
    Surpassed64BitLimit,
}
