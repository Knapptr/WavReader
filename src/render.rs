use crate::{chunk::ChunkData, riff::RiffChunk};
use textplots::{Chart, Plot, Shape};

impl RiffChunk{
    pub fn render_samples(&self){
        for chunk in &self.data{

            if let ChunkData::Data(data) = &chunk.data{
                let size = chunk.size;
                println!("Size: {} bytes", size);
                // print the left
                // let mut index = 0;
                let data: Vec<(f32,f32)> = data
                    .chunks_exact(2)
                    .enumerate()
                    .map(|(i,bytes)| (i as f32, i16::from_le_bytes([ bytes[0],bytes[1] ] ) as f32))
                    .collect();
                Chart::new(256, 64, 0.0, size as f32)
                    .lineplot(&Shape::Lines(&data))
                    .display();
}
            }
        }
}
