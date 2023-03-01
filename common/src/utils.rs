use substreams::Hex;

pub fn pretty_hex<T: std::convert::AsRef<[u8]>>(addr: &T) -> String {
    format!("0x{}", &Hex(addr).to_string())
}
