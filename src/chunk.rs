use crate::ChunkError;

#[derive(Debug)]
pub struct Chunk {
    pub id: String,
    pub size: u32,
    pub data: ChunkData
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
            "LIST" => ChunkData::List(ListChunk::try_from(value)?),
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
pub enum ChunkData{
    Riff { file_type: String, subchunks: Vec<Chunk>},
    List (ListChunk),
    Fmt(FmtChunk),
    Data(Vec<u8>),
    Unknown(Vec<u8>)
}

#[derive(Debug)]
pub struct ListChunk{
    pub list_type: String,
    pub data: ListData,
}
impl TryFrom<&[u8]> for ListChunk{
    type Error = ChunkError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 8 {return Err(ChunkError::SizeError()); }

        //get the id
        let id = str::from_utf8(&value[0..4])
            .map_err(|_| ChunkError::IdError())?
            .to_string();

        // confirm id
        if id != "LIST" {return Err(ChunkError::IdError())};

        // get size
        let size = u32::from_le_bytes(value[4..8]
            .try_into()
            .map_err(|_|ChunkError::SizeError())?);

        // get list type
        let list_type = str::from_utf8(&value[8..12])
            .map_err(|_| ChunkError::ListTypeError())?
            .to_string();

        // get data
        let data_end_i = size as usize + 8; // accounts for the size of the list_type field
        let data = match list_type.as_str(){
            "INFO" => ListData::Info(InfoData::try_from(&value[12..data_end_i]).map_err(|_| ChunkError::InfoError())?),
                _=> ListData::Other(value[12..data_end_i].to_vec())
        };

        return Ok(Self{
            list_type,
            data
        });
    }
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
        // let size = u32::from_le_bytes(value[4..8]
        //     .try_into()
        //     .map_err(|_|ChunkError::SizeError())?);

        
        // get format code
        let format_code = u16::from_le_bytes(value[8..10]
            .try_into()
            .map_err(|_|ChunkError::FormatCodeError())?);

        // check format code 
        match format_code{
            0x0001 => Ok(Self::Pcm( PCMFmt::try_from(&value[8..])
                    .map_err(|_|ChunkError::FormatCodeError() )?)),
            0xFFFE => Ok(Self::Extensible((ExtensibleFmt::try_from(&value[8..])
                        .map_err(|_| ChunkError::FormatCodeError())?))),
            _ => {println!("Formatcode: {}", format_code); unimplemented!()}
        }
    }
}

#[derive(Debug)]
struct PCMFmt{
    format_code: u16, // 1 
    number_of_channels: u16,
    sample_rate: u32,
    bytes_per_second: u32,
    block_align: u16,
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

        let block_align = u16::from_le_bytes(value[12..14]
            .try_into()
            .map_err(|_| ChunkError::BlockAlignError())?);

        let bits_per_sample = u16::from_le_bytes(value[14..16]
            .try_into()
            .map_err(|_| ChunkError::BitsPerSampleError())?);

        return Ok(Self{
            number_of_channels,
            sample_rate,
            bytes_per_second,
            bits_per_sample,
            block_align,
            format_code,
        })
    }
}
#[derive(Debug)]
struct ExtensibleFmt{
    format_code: u16, //0xFFFE or 65534
    number_of_channels: u16,
    sample_rate: u32,
    bytes_per_second: u32,
    block_align: u16,
    bits_per_sample: u16,
    cb_size: u16,
    valid_bits_per_sample: u16,
    channel_mask: u32,
    sub_format: [u8; 16],
}

impl TryFrom<&[u8]> for ExtensibleFmt{
    type Error = ChunkError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
       //get format code         
       let format_code = u16::from_le_bytes(value[0..2].try_into().map_err(|_| ChunkError::FormatCodeError())?);

       // get number of channels
       let number_of_channels = u16::from_le_bytes(value[2..4].try_into().map_err(|_| ChunkError::ChannelCountError())?);

       // get sample rate
       let sample_rate = u32::from_le_bytes(value[4..8].try_into().map_err(|_| ChunkError::SampleRateError())?);

       // get data rate
        let bytes_per_second = u32::from_le_bytes(value[8..12]
            .try_into()
            .map_err(|_|ChunkError::BytesPerSecondError())?);

        // //get black align
        let block_align = u16::from_le_bytes(value[12..14]
            .try_into()
            .map_err(|_| ChunkError::BlockAlignError())?);
        // get bits per sample
        let bits_per_sample = u16::from_le_bytes(value[14..16]
            .try_into()
            .map_err(|_| ChunkError::BitsPerSampleError())?);

        // get cb size
        let cb_size = u16::from_le_bytes(value[16..18]
            .try_into()
            .map_err(|_|ChunkError::CBSizeError())?);
            //
        let valid_bits_per_sample = u16::from_le_bytes(value[18..20]
            .try_into()
            .map_err(|_|ChunkError::ValidBitsError())?);

        // get channel mask
        let channel_mask = u32::from_le_bytes(value[20..24]
            .try_into()
            .map_err(|_| ChunkError::ChannelMaskError())?);

        // get subformat
        let sub_format :[u8;16] = value[24..40]
            .try_into()
            .map_err(|_| ChunkError::ChannelMaskError())?;

        return Ok(Self{
            sample_rate,
            sub_format,
            channel_mask,
            number_of_channels,
            valid_bits_per_sample,
            cb_size,
            bytes_per_second,
            bits_per_sample,
            format_code,
            block_align,

        })
    }
}

#[derive(Debug)]
pub enum ListData{
    Info(InfoData),
    Other(Vec<u8>)

}
#[derive(Debug)]
struct InfoData{
    iarl: Option<String>, // The location where the subject of the file is archived
    iart: Option<String>, // The artist of the original subject of the file
    icms: Option<String>, // The name of the person or organization that commissioned the original subject of the file
    icmt: Option<String>, // General comments about the file or its subject
    icop: Option<String>, // Copyright information about the file (e.g., "Copyright Some Company 2011")
    icrd: Option<String>, // The date the subject of the file was created (creation date) (e.g., "2022-12-31")
    icrp: Option<String>, // Whether and how an image was cropped
    idim: Option<String>, // The dimensions of the original subject of the file
    idpi: Option<String>, // Dots per inch settings used to digitize the file
    ieng: Option<String>, // The name of the engineer who worked on the file
    ignr: Option<String>, // The genre of the subject
    ikey: Option<String>, // A list of keywords for the file or its subject
    ilgt: Option<String>, // Lightness settings used to digitize the file
    imed: Option<String>, // Medium for the original subject of the file
    inam: Option<String>, // Title of the subject of the file (name)
    iplt: Option<String>, // The number of colors in the color palette used to digitize the file
    iprd: Option<String>, // Name of the title the subject was originally intended for
    isbj: Option<String>, // Description of the contents of the file (subject)
    isft: Option<String>, // Name of the software package used to create the file
    isrc: Option<String>, // The name of the person or organization that supplied the original subject of the file
    isrf: Option<String>, // The original form of the material that was digitized (source form)
    itch: Option<String>, // The name of the technician who digitized the subject file
}
impl InfoData{
    fn initialize()->Self{
        return Self{
            iarl: None,
            iart: None,
            icms: None,
            icmt: None,
            icop: None,
            icrd: None,
            icrp: None,
            idim: None,
            idpi: None,
            ieng: None,
            ignr: None,
            ikey: None,
            ilgt: None,
            imed: None,
            inam: None,
            iplt: None,
            iprd: None,
            isbj: None,
            isft: None,
            isrc: None,
            isrf: None,
            itch: None,
        }
    }
}

impl TryFrom<&[u8]> for InfoData{
    type Error = ChunkError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
       let mut data = InfoData::initialize();
       let mut index = 0;
       while index < value.len(){
           // get infoid
           let info_id = str::from_utf8(&value[index..index + 4]).map_err(|_| ChunkError::IdError())?.to_string();
           // get infoSize
           let info_size = u32::from_le_bytes(value[index + 4 .. index + 8].try_into().map_err(|_|ChunkError::SizeError())?);
           // get info
           let info_string = str::from_utf8(&value[index + 8.. index + 8 + info_size as usize - 1]).map_err(|_| ChunkError::InfoError())?.to_string(); // - 1 is the null char
           // handle padding
           let padding = info_size % 2;
           // set property on data to Some
           match info_id.as_str(){
            "IARL" => data.iarl = Some(info_string),
            "IART" => data.iart = Some(info_string),
            "ICMS" => data.icms = Some(info_string),
            "ICMT" => data.icmt = Some(info_string),
            "ICOP" => data.icop = Some(info_string),
            "ICRD" => data.icrd = Some(info_string),
            "ICRP" => data.icrp = Some(info_string),
            "IDIM" => data.idim = Some(info_string),
            "IDPI" => data.idpi = Some(info_string),
            "IENG" => data.ieng = Some(info_string),
            "IGNR" => data.ignr = Some(info_string),
            "IKEY" => data.ikey = Some(info_string),
            "ILGT" => data.ilgt = Some(info_string),
            "IMED" => data.imed = Some(info_string),
            "INAM" => data.inam = Some(info_string),
            "IPLT" => data.iplt = Some(info_string),
            "IPRD" => data.iprd = Some(info_string),
            "ISBJ" => data.isbj = Some(info_string),
            "ISFT" => data.isft = Some(info_string),
            "ISRC" => data.isrc = Some(info_string),
            "ISRF" => data.isrf = Some(info_string),
            "ITCH" => data.itch = Some(info_string),
            _ => unimplemented!(),
           }
           // advance index
           index += 8 + info_size as usize + padding as usize;
       }
       Ok(data)
    }
}
