//! A library for parsing Source 2013 demo files, built on top of the `nom` parser combinator library.

use nom::{
    bytes::streaming::{tag, take},
    combinator::map,
    number::streaming::{le_f32, le_u32},
    sequence::tuple,
    IResult,
};

/// The header of a Source 2013 demo file.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    demo_protocol: u32,
    network_protocol: u32,
    server_name: String,
    client_name: String,
    map_name: String,
    game_directory: String,
    playback_time: f32,
    playback_ticks: u32,
    playback_frames: u32,
    signon_length: u32,
}

/// Parse a null-terminated string from the input, with a maximum length.
fn pathstr(input: &[u8]) -> IResult<&[u8], String> {
    let (input, bytes) = take(260usize)(input)?;
    let bytes = bytes.split(|&b| b == 0).next().unwrap();
    Ok((input, String::from_utf8_lossy(bytes).to_string()))
}

// Combinator for parsing a demo header.
fn header<'a>(input: &'a [u8]) -> IResult<&'a [u8], Header> {
    map(
        tuple((
            tag("HL2DEMO\0"),
            le_u32,
            le_u32,
            pathstr,
            pathstr,
            pathstr,
            pathstr,
            le_f32,
            le_u32,
            le_u32,
            le_u32,
        )),
        |(
            _,
            demo_protocol,
            network_protocol,
            server_name,
            client_name,
            map_name,
            game_directory,
            playback_time,
            playback_ticks,
            playback_frames,
            signon_length,
        )| Header {
            demo_protocol,
            network_protocol,
            server_name,
            client_name,
            map_name,
            game_directory,
            playback_time,
            playback_ticks,
            playback_frames,
            signon_length,
        },
    )(input)
}

/// A frame of a Source 2013 demo file.
pub struct Frame {
    pub server_frame: u32,
    pub client_frame: u32,
    pub sub_packet_size: u32,
    pub buffer: Vec<u8>,
}

/// Parse a frame from the input.
fn frame(input: &[u8]) -> IResult<&[u8], Frame> {
    println!("frame");
    let (input, (server_frame, client_frame, sub_packet_size)) =
        tuple((le_u32, le_u32, le_u32))(input)?;
    let (input, buffer) = take(sub_packet_size)(input)?;
    Ok((
        input,
        Frame {
            server_frame,
            client_frame,
            sub_packet_size,
            buffer: buffer.to_vec(),
        },
    ))
}

/// Parse a sequence of frames from the input.
fn frames(input: &[u8], count: usize) -> IResult<&[u8], Vec<Frame>> {
    let mut frames = Vec::with_capacity(count);
    let mut input = input;
    for _ in 0..count {
        let (_, _) = le_u32::<_, nom::error::Error<&[u8]>>(input).unwrap();
        let (new_input, frame) = frame(input)?;
        frames.push(frame);
        input = new_input;
    }
    Ok((input, frames))
}

/// A demo file.
pub struct Demo {
    pub header: Header,
    pub frames: Vec<Frame>,
}

/// Parse a demo file from the input.
pub fn demo(input: &[u8]) -> IResult<&[u8], Demo> {
    let (input, header) = header(input)?;
    let (input, frames) = frames(input, header.playback_frames as usize)?;
    Ok((input, Demo { header, frames }))
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_demo0() {
        let input = fs::read("assets/test0.dem").expect("Failed to read file");
        let (_, demo) = super::demo(&input).expect("error");
    }
}
