use crate::binary_io::IOAble;

#[derive(Debug, Clone)]
pub struct Grid {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Option<(String, u8)>>,
}

// static mut EMPTY: Option<(String, u8)> = None;

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        Grid {
            width,
            height,
            cells: vec![None; width as usize * height as usize],
        }
    }

    pub fn get(&mut self, x: u16, y: u16) -> &mut Option<(String, u8)> {
        // if x >= self.width || y >= self.height {
        //     return &mut EMPTY;
        // }
        &mut self.cells[(y * self.width + x) as usize]
    }
}

impl IOAble for Grid {
    fn read_from(stream: &mut crate::binary_io::InputStream) -> Option<Self> {
        Some(Grid {
            width: stream.read()?,
            height: stream.read()?,
            cells: {
                let len = stream.read::<u32>()?;
                let mut cells = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let id = stream.read::<String>()?;
                    let direction = stream.read::<u8>()?;
                    cells.push(if id.is_empty() { None } else { Some((id, direction)) });
                }
                cells
            }
        })
    }

    fn write_to(&self, stream: &mut crate::binary_io::OutputStream) {
        stream.write(self.width);
        stream.write(self.height);
        stream.write(self.cells.len() as u32);
        for cell in &self.cells {
            if let Some((id, direction)) = cell {
                stream.write(id);
                stream.write(direction);
            }
            else {
                stream.write("");
                stream.write(0u8);
            }
        }
    }
}
