pub type HexaFreezeResult<T> = Result<T, HexaFreezeError>;

#[derive(Debug, PartialEq, Eq)]
pub enum HexaFreezeError {
    EpochInTheFuture,
    EpochTooFarInThePast,
    NodeIdTooLarge,
    ClockWentToTheFuture,
    Surpassed64BitLimit,
}
