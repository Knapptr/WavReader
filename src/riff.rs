use crate::chunk::Chunk;
use crate::errors::ChunkError;

#[derive(Debug)]
pub struct RiffChunk{
   pub id: String,
   pub size: u32,
   pub file_type: String,
   pub data: Vec<Chunk>
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
            let padding = (chunk.size as usize) % 2 ;
            index += (chunk.size as usize) + 8 + padding;
            // add chunk to list
            data.push(chunk);
       }
       return Ok(
           RiffChunk {id,data,size,file_type }
       )
   }
}



