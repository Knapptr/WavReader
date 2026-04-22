
#[derive(Debug)]
pub enum ChunkError{
    RiffError(),
    IdError(),
    FormatCodeError(),
    SizeError(),
    ChannelCountError(),
    SampleRateError(),
    BytesPerSecondError(),
    CBSizeError(),
    BitsPerSampleError(),
    ChannelMaskError(),
    BlockAlignError(),
    ValidBitsError(),
    ListTypeError(),
    InfoError(),
}
