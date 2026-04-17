const RIFF: [u8;4]  = *b"RIFF";

fn get_file_bytes(path:&str)-> std::io::Result<Vec<u8>>{
    std::fs::read(path)
}


#[derive(Debug)]
struct Chunk {
    id: String,
    size: u32,
    data: Vec<u8>
}
fn create_chunks(data: &[u8])->Result<Vec<Chunk>,String>{
    // read id (4 bytes)
    let mut index:usize = 0;
    let mut chunks: Vec<Chunk> = Vec::new();
    while index < data.len() {
        // read chunk id
        let chunk_id = str::from_utf8(&data[index..index + 4]).unwrap(); //todo: handle an unsuccesful conversion
        index += 4;
        let chunk_size = u32::from_le_bytes(data[index..index+4].try_into().unwrap());
        index += 4;
        let chunk_data = data[index..index+chunk_size as usize].to_vec();
        chunks.push(Chunk{
            id: chunk_id.to_string(),
            size: chunk_size,
            data: chunk_data
        });
        index += chunk_size as usize;
        if chunk_size % 2 != 0 { index += 1;} // handle padding bytes
    }
    return Ok(chunks);
}

fn main(){
    let snore_path = "/home/tknapp/snore.wav";
    if let Ok(bytes) = get_file_bytes(snore_path){
       // check that it is a RIFF 
        if bytes[0..4] != RIFF{
            eprintln!("Not a RIFF file");
            return;
        }
        let file_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) + 8;
        let format = &bytes[8..12];
        if format != b"WAVE"{ eprintln!("Not a WAV file"); return;}
        println!("File size: {} bytes", file_size);
        let unprocessed_chunks = bytes[12..].to_vec();
        let chunks = create_chunks(&unprocessed_chunks).unwrap();
        for ck in chunks{
            println!("chunk id: {}", ck.id);
        }


    }else{
        eprintln!("Could not load file");
    }
}
