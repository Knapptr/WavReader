const RIFF: [u8;4]  = *b"RIFF";

fn get_file_bytes(path:&str)-> std::io::Result<Vec<u8>>{
    std::fs::read(path)
}


#[derive(Debug)]
struct RiffChunk{
    id: String,
    size: u32,
    file_type: String,
    data: Vec<Chunk>
}
impl TryFrom<&[u8]> for RiffChunk{
   type Error = ChunkError; 
   fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
       // read the riff header
       let id = str::from_utf8(&value[0..4])
           .map_err(|_| ChunkError::RiffError())?.to_string();
       if id != "RIFF" { return Err(ChunkError::RiffError())}

       //read the size
       let size = u32::from_le_bytes(value[4..8]
           .try_into()
           .map_err(|_| ChunkError::SizeError())?);

       // read the file format
       let file_type = str::from_utf8(&value[8..12]).map_err(|_| ChunkError::RiffError())?.to_string();

       // try to iterate through data and create chunks
       let mut index: usize = 12;
       let mut data = Vec::new();
       while index < value.len(){
            let chunk = Chunk::try_from(&value[index..])?;
            //move to next chunk
            index += (chunk.size as usize) + 8;
            // add chunk to list
            data.push(chunk);
       }
       return Ok(
           RiffChunk {id,data,size,file_type }
       )
   }
}

#[derive(Debug)]
struct Chunk {
    id: String,
    size: u32,
    data: ChunkData
}

#[derive(Debug)]
enum ChunkError{
    RiffError(),
    IdError(),
    FormatCodeError(),
    SizeError(),
    ChannelCountError(),
    SampleRateError(),
    BytesPerSecondError(),
    BitsPerSampleError()
}
impl TryFrom<&[u8]> for Chunk{
    type Error = ChunkError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // make sure there are at least 8 bytes
        if value.len() < 8 { return Err(ChunkError::SizeError())}
        // get header/id
        let id = str::from_utf8(&value[0..4]).map_err(|_|ChunkError::IdError())?.to_string();

        // get size
        let size = u32::from_le_bytes(value[4..8].try_into().map_err(|_| ChunkError::SizeError())?);

        // get data
        // make sure size is valid
        if value.len() < (size + 8) as usize{
            return Err(ChunkError::SizeError())
        }
        // create data vec
        let raw_data = &value[8.. (8 + size as usize)];
        let data:ChunkData = match id.as_str(){
            "data" =>  ChunkData::Data(raw_data.to_vec()),
            "fmt " => ChunkData::Fmt(
                FmtChunk::try_from(value)?
            ),
            _ => ChunkData::Unknown(raw_data.to_vec())
        };
        // let data = value[8..(8 + size as usize)].to_vec();

        return Ok(Self{
            id,
            size,
            data
        })
    }
}

#[derive(Debug)]
enum ChunkData{
    Riff { file_type: String, subchunks: Vec<Chunk>},
    List { list_type: String, subchunks: Vec<Chunk>},
    Fmt(FmtChunk),
    Data(Vec<u8>),
    Unknown(Vec<u8>)
}

#[derive(Debug)]
enum FmtChunk{
    Pcm(PCMFmt),// uses format code 0x0001
    Extensible(ExtensibleFmt), // uses format code 0xFFFE
    Unknown{fmt_code:u16, data: Vec<u8>}
}
impl TryFrom<&[u8]> for FmtChunk{
    type Error = ChunkError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // check the size
        if value.len() < 8 {return Err(ChunkError::SizeError())}

        // get the id
        let id = str::from_utf8(&value[0..4])
            .map_err(|_| ChunkError::IdError())?
            .to_string();
        // make sure the id is "fmt "
        if id != "fmt " { return Err(ChunkError::IdError())}

        // get size
        let size = u32::from_le_bytes(value[4..8]
            .try_into()
            .map_err(|_|ChunkError::SizeError())?);

        
        // get format code
        let format_code = u16::from_le_bytes(value[8..10]
            .try_into()
            .map_err(|_|ChunkError::FormatCodeError())?);

        // check format code 
        match format_code{
            0x0001 => Ok(Self::Pcm( PCMFmt::try_from(&value[8..])
                    .map_err(|_|ChunkError::FormatCodeError() )?)),
            _ => unimplemented!()
        }
    }
}

#[derive(Debug)]
struct PCMFmt{
    format_code: u16, // 1 
    number_of_channels: u16,
    sample_rate: u32,
    bytes_per_second: u32,
    bits_per_sample: u16,
}
impl TryFrom<&[u8]> for PCMFmt{
    type Error = ChunkError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // check format code
        let format_code = u16::from_le_bytes(value[0..2]
            .try_into()
            .map_err(|_| ChunkError::FormatCodeError())?);
        if format_code != 0x0001 { return Err(ChunkError::FormatCodeError())}
        
        // get data
        let number_of_channels = u16::from_le_bytes(value[2..4]
           .try_into()
           .map_err(|_| ChunkError::ChannelCountError())?);

        let sample_rate = u32::from_le_bytes(value[4..8]
            .try_into()
            .map_err(|_| ChunkError::SampleRateError())?);

        let bytes_per_second = u32::from_le_bytes(value[8..12]
            .try_into()
            .map_err(|_|ChunkError::BytesPerSecondError())?);

        let bits_per_sample = u16::from_le_bytes(value[12..14]
            .try_into()
            .map_err(|_| ChunkError::BitsPerSampleError())?);

        return Ok(Self{
            number_of_channels,
            sample_rate,
            bytes_per_second,
            bits_per_sample,
            format_code
        })
    }
}
#[derive(Debug)]
struct ExtensibleFmt{
    format_code: u16, //0xFFFE or 65534
    number_of_channels: u16,
    sample_rate: u32,
    bytes_per_second: u32,
    bits_per_sample: u16,
    extra_param_size: u16,
    extra_params: u32, // this field is u32, but maybe could be more depending on the extra_param_size? It is hard to find information about the extensible fmt
}


fn main(){
    let snore_path = "/home/tknapp/snore.wav";
    let bytes = get_file_bytes(snore_path).unwrap();
    let riff = RiffChunk::try_from(&bytes[0..]).unwrap();
    for chunk in riff.data{
        println!("{}", chunk.id);
    }
}
