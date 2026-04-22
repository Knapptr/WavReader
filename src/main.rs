mod riff;
mod render;
mod errors;
mod chunk;
use riff::{RiffChunk};
use chunk::ChunkData;
use errors::ChunkError;
const RIFF: [u8;4]  = *b"RIFF";


fn get_file_bytes(path:&str)-> std::io::Result<Vec<u8>>{
    std::fs::read(path)
}

#[derive(Debug)]
struct Wav<T>{
    sample_rate: u32,
    channels: u32,
    bit_depth: u32,
    bytes: u32,
    data: Vec<T>
}

enum WavError{
    ParseError()
}

impl TryFrom<RiffChunk> for Wav<i16>{
    type Error = WavError;
    fn try_from(value: RiffChunk) -> Result<Self, Self::Error> {
        // TODO iterate through riffchunk and pull out nesc data
        todo!()
    }
}



fn main(){
    // let sample_path = "media/BT0A0D3.WAV";
    let sample_path = "media/foat.wav";
    let bytes = get_file_bytes(sample_path).unwrap();
    let riff = RiffChunk::try_from(&bytes[0..]).unwrap();
    for chunk in &riff.data{
        println!("Chunk Type: {}", chunk.id);
        if let ChunkData::List(list_chunk) = &chunk.data{
            println!("{:?}", list_chunk.data);
        }
    }
    riff.render_samples();
}
