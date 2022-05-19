use crate::{binary_io::{OutputStream, InputStream}, grid::Grid};

#[derive(Debug, Clone)]
pub enum JMMessage {
    GetGrid,
    SetGrid(Grid),
    SetCell(u16, u16, String, u8),
    Delete(u16, u16),
}

impl JMMessage {
    pub fn write_v1(&self, stream: &mut OutputStream) {
        match self {
            JMMessage::GetGrid => {
                stream.write(0u8);
            },
            JMMessage::SetGrid(grid) => {
                stream.write(1u8);
                stream.write(grid);
            },
            JMMessage::SetCell(x, y, id, direction) => {
                stream.write(2u8);
                stream.write(*x);
                stream.write(*y);
                stream.write(id);
                stream.write(*direction);
            },
            JMMessage::Delete(x, y) => {
                stream.write(3u8);
                stream.write(*x);
                stream.write(*y);
            },
        }
    }

    pub fn parse_v1(stream: &mut InputStream) -> Option<JMMessage> {
        match stream.read::<u8>()? {
            0 => Some(JMMessage::GetGrid),
            1 => Some(JMMessage::SetGrid(stream.read::<Grid>()?)),
            2 => Some(JMMessage::SetCell(
                /*x*/ stream.read::<u16>()?,
                /*y*/ stream.read::<u16>()?,
                /*id*/ stream.read::<String>()?,
                /*dir*/ stream.read::<u8>()?
            )),
            3 => Some(JMMessage::Delete(
                stream.read::<u16>()?,
                stream.read::<u16>()?
            )),
            _ => None
        }
    }
}
