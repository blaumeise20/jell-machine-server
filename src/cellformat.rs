use std::collections::HashMap;

use crate::binary_io::InputStream;

struct Parsed {
    width: u16,
    height: u16,
    celltable: HashMap<u16, String>,
    cells: Vec<(u32, u16)>,
    directions: Vec<(u32, u8)>,
}

fn parse(data: Vec<u8>) -> Option<Parsed> {

    // header:
    //               (width u16) wwwwwwww wwwwwwww
    //              (height u16) hhhhhhhh hhhhhhhh
    //    (celltable length u15) 0lllllll llllllll
    // as many as celltable length:
    //      (cell id length u32) iiiiiiii iiiiiiii iiiiiiii iiiiiiii
    //       (cell id bytes  u8) bbbbbbbb...
    // cell map:
    //     (cell map length u64) llllllll llllllll llllllll llllllll
    //                           llllllll llllllll llllllll llllllll
    //    multiple:
    //       (repeat count? u31) 1rrrrrrr rrrrrrrr rrrrrrrr rrrrrrrr
    //                 (empty 0) 00000000
    //          (or cell id u15) 0iiiiiii iiiiiiii




    let mut stream = InputStream::new(data);
    let width = stream.read()?;
    let height = stream.read()?;

    let celltable_length = stream.read()?;
    let mut celltable = HashMap::with_capacity(celltable_length as usize);
    if celltable_length > 2 << 15 { return None; }
    for i in 0..celltable_length {
        celltable.insert(i, stream.read()?);
    }

    let cells_length = stream.read::<u64>()?;
    let mut cells = Vec::with_capacity(cells_length as usize);
    for _ in 0..cells_length {
        cells.push((stream.read()?, stream.read()?));
    }


    Some(Parsed {
        width,
        height,
        celltable,
        cells,
        directions: Vec::new(),
    })
}
