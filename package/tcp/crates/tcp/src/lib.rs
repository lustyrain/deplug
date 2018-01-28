extern crate byteorder;
extern crate libc;

#[macro_use]
extern crate plugkit;

use std::io::{Cursor, Error, ErrorKind, BufRead};
use byteorder::BigEndian;
use plugkit::reader::ByteReader;
use plugkit::layer;
use plugkit::layer::{Confidence, Layer};
use plugkit::context::Context;
use plugkit::worker::Worker;
use plugkit::variant::Value;
use plugkit::attr::ResultValue;
use plugkit::token;

struct TCPWorker {}

impl Worker for TCPWorker {
    fn analyze(&self, ctx: &mut Context, layer: &mut Layer) -> Result<(), Error> {
        let (slice, payload_range) = {
            let payload = layer
                .payloads()
                .next()
                .ok_or(Error::new(ErrorKind::Other, "no payload"))?;
            let slice = payload
                .slices()
                .next()
                .ok_or(Error::new(ErrorKind::Other, "no slice"))?;

            (slice, payload.range())
        };

        let (src_addr, dst_addr) = {
            let src : &[u8] = layer.attr(token::join(layer.id(), token!(".src"))).unwrap().get();
            let dst : &[u8] = layer.attr(token::join(layer.id(), token!(".dst"))).unwrap().get();
            (src, dst)
        };

        let child = layer.add_layer(ctx, token!("tcp"));
        child.set_confidence(Confidence::Error);
        child.add_tag(token!("tcp"));
        child.set_range(&payload_range);

        let mut rdr = Cursor::new(slice);
        let (src, src_range) = ByteReader::read_u16::<BigEndian>(&mut rdr)?;
        let (dst, dst_range) = ByteReader::read_u16::<BigEndian>(&mut rdr)?;

        {
            let attr = child.add_attr(ctx, token!("tcp.src"));
            attr.set(&src);
            attr.set_range(&src_range);
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.dst"));
            attr.set(&dst);
            attr.set_range(&dst_range);
        }

        let worker = src as u32 + dst as u32  +
            src_addr.iter().fold(0, |acc, &x| acc + x as u32) +
            dst_addr.iter().fold(0, |acc, &x| acc + x as u32);
        child.set_worker((worker % layer::MAX_WORKER as u32) as u8);

        {
            let attr = child.add_attr(ctx, token!("tcp.seq"));
            attr.set_result(ByteReader::read_u32::<BigEndian>(&mut rdr))?;
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.ack"));
            attr.set_result(ByteReader::read_u32::<BigEndian>(&mut rdr))?;
        }

        let (ofs_and_flag, range) = ByteReader::read_u8(&mut rdr)?;
        let data_offset = ofs_and_flag as u32 >> 4;
        {
            let attr = child.add_attr(ctx, token!("tcp.offset"));
            attr.set(&data_offset);
            attr.set_range(&range);
        }
        let (flag, _) = ByteReader::read_u8(&mut rdr)?;
        let flags = flag | ((ofs_and_flag & 0x1) << 8);
        {
            let attr = child.add_attr(ctx, token!("tcp.flags"));
            attr.set(&flags);
            attr.set_range(&(12..14));
            attr.set_typ(token!("@flags"));
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.flags.ns"));
            attr.set(&((flags & (0x1 << 8)) != 0));
            attr.set_range(&(12..13));
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.flags.cwr"));
            attr.set(&((flags & (0x1 << 7)) != 0));
            attr.set_range(&(13..14));
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.flags.ece"));
            attr.set(&((flags & (0x1 << 6)) != 0));
            attr.set_range(&(13..14));
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.flags.urg"));
            attr.set(&((flags & (0x1 << 5)) != 0));
            attr.set_range(&(13..14));
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.flags.ack"));
            attr.set(&((flags & (0x1 << 4)) != 0));
            attr.set_range(&(13..14));
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.flags.psh"));
            attr.set(&((flags & (0x1 << 3)) != 0));
            attr.set_range(&(13..14));
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.flags.rst"));
            attr.set(&((flags & (0x1 << 2)) != 0));
            attr.set_range(&(13..14));
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.flags.syn"));
            attr.set(&((flags & (0x1 << 1)) != 0));
            attr.set_range(&(13..14));
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.flags.fin"));
            attr.set(&((flags & (0x1 << 0)) != 0));
            attr.set_range(&(13..14));
        }

        {
            let attr = child.add_attr(ctx, token!("tcp.window"));
            attr.set_result(ByteReader::read_u16::<BigEndian>(&mut rdr))?;
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.checksum"));
            attr.set_result(ByteReader::read_u16::<BigEndian>(&mut rdr))?;
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.urgent"));
            attr.set_result(ByteReader::read_u16::<BigEndian>(&mut rdr))?;
        }
        {
            let attr = child.add_attr(ctx, token!("tcp.options"));
            attr.set_typ(token!("@nested"));
        }

        loop {
            let (typ, range) = ByteReader::read_u8(&mut rdr)?;
            match typ {
                1 => {
                    let attr = child.add_attr(ctx, token!("tcp.options.nop"));
                    attr.set(&true);
                    attr.set_range(&range);
                    attr.set_typ(token!("@novalue"));
                }
                2 => {
                    rdr.consume(1);
                    let attr = child.add_attr(ctx, token!("tcp.options.mss"));
                    attr.set_result(ByteReader::read_u16::<BigEndian>(&mut rdr))?;
                }
                3 => {
                    rdr.consume(1);
                    let attr = child.add_attr(ctx, token!("tcp.options.scale"));
                    attr.set_result(ByteReader::read_u8(&mut rdr))?;
                }
                4 => {
                    rdr.consume(1);
                    let attr = child.add_attr(ctx, token!("tcp.options.selectiveAckPermitted"));
                    attr.set(&true);
                    attr.set_range(&range);
                }
                _ => break
            }
        }

        let option_data_offset = data_offset * 4;
        {
            let (data, range) = ByteReader::read_slice(&mut rdr, option_data_offset as usize)?;
            let payload = child.add_payload(ctx);
            let offset = payload_range.start;
            payload.add_slice(data);
            payload.set_range(&(range.start + offset..range.end + offset));
        }

        child.set_confidence(Confidence::Exact);
        Ok(())
    }
}

plugkit_module!({});
plugkit_api_layer_hints!(token!("[tcp]"));
plugkit_api_worker!(TCPWorker, TCPWorker {});
