extern crate pcap;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate genet_sdk;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

use genet_sdk::{
    attr::{Attr, AttrBuilder, AttrClass},
    context::Context,
    io::{Reader, ReaderWorker},
    layer::{Layer, LayerBuilder, LayerClass},
    ptr::Ptr,
    result::Result,
    slice::Slice,
    token,
    variant::Variant,
};
use pcap::Header;

use std::{
    io::{BufRead, BufReader, Error, ErrorKind, Read},
    mem,
    process::{Child, ChildStdout, Command, Stdio},
    slice,
};

#[derive(Deserialize)]
struct Arg {
    cmd: String,
    args: Vec<String>,
    link: u32,
}

#[derive(Clone)]
struct PcapReader {}

impl Reader for PcapReader {
    fn new_worker(&self, ctx: &Context, arg: &str) -> Result<Box<ReaderWorker>> {
        let arg: Arg = serde_json::from_str(arg)?;
        let mut child = Command::new(&arg.cmd)
            .args(&arg.args)
            .stdout(Stdio::piped())
            .spawn()?;
        let reader = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| Error::new(ErrorKind::Other, "no stdout"))?,
        );
        let link_class = LayerBuilder::new(format!("[link-{}]", arg.link))
            .header(Attr::with_value(&TYPE_CLASS, 0..0, arg.link as i64))
            .build();
        Ok(Box::new(PcapReaderWorker {
            child,
            reader,
            link_class,
        }))
    }

    fn id(&self) -> &str {
        "pcap"
    }
}

struct PcapReaderWorker {
    child: Child,
    reader: BufReader<ChildStdout>,
    link_class: Ptr<LayerClass>,
}

impl ReaderWorker for PcapReaderWorker {
    fn read(&mut self) -> Result<Vec<Layer>> {
        let mut header = String::new();
        self.reader.read_line(&mut header)?;
        let header = header.trim();
        if header.is_empty() {
            return Ok(vec![]);
        }
        let header: Header = serde_json::from_str(header)?;
        let mut data = vec![0u8; header.datalen as usize];
        self.reader.read_exact(&mut data)?;
        let payload = Slice::from(data);
        let mut layer = Layer::new(&self.link_class, payload);
        layer.add_attr(Attr::with_value(&LENGTH_CLASS, 0..0, header.actlen as u64));
        layer.add_attr(Attr::with_value(
            &TS_CLASS,
            0..0,
            header.ts_sec as f64 + header.ts_usec as f64 / 1000_000f64,
        ));
        layer.add_attr(Attr::with_value(&TS_SEC_CLASS, 0..0, header.ts_sec as u64));
        layer.add_attr(Attr::with_value(
            &TS_USEC_CLASS,
            0..0,
            header.ts_usec as u64,
        ));
        Ok(vec![layer])
    }
}

impl Drop for PcapReaderWorker {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

lazy_static! {
    static ref TYPE_CLASS: Ptr<AttrClass> = AttrBuilder::new("link.type").build();
    static ref LENGTH_CLASS: Ptr<AttrClass> = AttrBuilder::new("link.length").build();
    static ref TS_CLASS: Ptr<AttrClass> = AttrBuilder::new("link.timestamp")
        .typ("@datetime:unix")
        .build();
    static ref TS_SEC_CLASS: Ptr<AttrClass> = AttrBuilder::new("link.timestamp.sec").build();
    static ref TS_USEC_CLASS: Ptr<AttrClass> = AttrBuilder::new("link.timestamp.usec").build();
}

genet_readers!(PcapReader {});
