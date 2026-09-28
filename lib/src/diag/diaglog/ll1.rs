use deku::prelude::*;

/// Qualcomm emits several incompatible layouts under log code 0xb114, tagged by
/// a leading version byte. Upstream implements version 1 and asserts on it,
/// which turns every other version into a hard parse failure for the whole
/// message.
///
/// The Inseego M2100 (Qualcomm SDX55) emits version 161. In one 16-hour capture
/// that was 29,332 failures -- 98.9% of every skipped message -- producing a
/// 186MB error log from a 14.9MB source and filling the device's rootfs. No
/// analyser consumes this log type, so the entire cost was noise.
///
/// An unrecognised version is now kept as opaque bytes rather than failing: it
/// leaves the error path, and the payload is retained in case the layout is
/// ever worked out.
#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "hdr_len: u16")]
pub struct ServingCellTiming {
    pub version: u8,
    #[deku(ctx = "*version, hdr_len.saturating_sub(1)")]
    pub body: ServingCellTimingBody,
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "version: u8, len: u16", id = "version")]
pub enum ServingCellTimingBody {
    #[deku(id = "1")]
    V1(V1),
    /// Unknown layout, not unknown data -- keep the bytes.
    #[deku(id_pat = "_")]
    Unsupported {
        #[deku(count = "len")]
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(bit_order = "lsb")]
pub struct V1 {
    #[deku(bits = 5, assert = "(1..=20).contains(num_records)")]
    pub num_records: u8,
    #[deku(bits = 4, assert = "*starting_sub_fn <= 9")]
    pub starting_sub_fn: u8,
    #[deku(
        bits = 10,
        pad_bits_after = "5",
        assert = "*starting_system_fn <= 1023"
    )]
    pub starting_system_fn: u16,
    #[deku(
        bits = 19,
        pad_bits_after = "13",
        assert = "*starting_dl_frame_timing_offs <= 307200"
    )]
    pub starting_dl_frame_timing_offs: u32, // in Ts units
    #[deku(bits = 19, assert = "*starting_ul_frame_timing_offs <= 307200")]
    pub starting_ul_frame_timing_offs: u32, // in Ts units
    #[deku(bits = 11, pad_bits_after = "2")]
    pub starting_ul_timing_advance: u16, // in 16 Ts units
    #[deku(count = "num_records")]
    pub timing_adjustment: Vec<TimingAdjustment>,
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(bit_order = "lsb", ctx = "_: deku::ctx::Order")]
pub struct TimingAdjustment {
    #[deku(
        bits = 11,
        assert = "(-512..=511).contains(dl_frame_timing_adjustment)"
    )]
    pub dl_frame_timing_adjustment: i16, // in Ts units
    #[deku(bits = 5, assert = "(-16..=15).contains(ul_frame_timing_adjustment)")]
    pub ul_frame_timing_adjustment: i8, // in Ts units
    #[deku(assert = "(-128..=127).contains(timing_advance)")]
    pub timing_advance: i8, // in 16 Ts units
}
