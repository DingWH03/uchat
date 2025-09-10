// src/frame.rs
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

pub const MAGIC: &[u8; 2] = b"IM";
pub const VERSION: u8 = 1;

/// 方向：服务端只发送 S2C，只接收 C2S
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction { C2S = 0, S2C = 1 }

impl TryFrom<u8> for Direction {
    type Error = FrameError;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v { 0 => Ok(Direction::C2S), 1 => Ok(Direction::S2C), x => Err(FrameError::InvalidDirection(x)) }
    }
}

#[derive(Debug)]
pub enum FrameError {
    InvalidMagic([u8; 2]),
    InvalidVersion(u8),
    InvalidDirection(u8),
    Truncated,
    Utf8(std::string::FromUtf8Error),
    Io(std::io::Error),
    InvalidKind(u8), // 由各类型在 from_bytes 时决定是否使用
}
impl From<std::io::Error> for FrameError { fn from(e: std::io::Error) -> Self { Self::Io(e) } }
impl From<std::string::FromUtf8Error> for FrameError { fn from(e: std::string::FromUtf8Error) -> Self { Self::Utf8(e) } }

/// 帧头（固定）：
/// [ magic(2)="IM" | version(1)=1 | dir(1) | kind(1) | flags(2) | payload_len(4) | payload ... ]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub dir: Direction,
    pub kind_u8: u8,   // 各消息自定义意义空间
    pub flags: u16,    // 预留（压缩/加密/分片等）
    pub payload_len: usize,
}

fn write_header(out: &mut Vec<u8>, dir: Direction, kind_u8: u8, flags: u16, payload_len: usize) -> Result<(), FrameError> {
    if payload_len > u32::MAX as usize { return Err(FrameError::Truncated); }
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.push(dir as u8);
    out.push(kind_u8);
    out.write_u16::<BigEndian>(flags)?;
    out.write_u32::<BigEndian>(payload_len as u32)?;
    Ok(())
}

fn read_header(c: &mut Cursor<&[u8]>) -> Result<Header, FrameError> {
    let mut mg = [0u8; 2];
    c.read_exact(&mut mg)?;
    if &mg != MAGIC { return Err(FrameError::InvalidMagic(mg)); }

    let ver = c.read_u8()?;
    if ver != VERSION { return Err(FrameError::InvalidVersion(ver)); }

    let dir = Direction::try_from(c.read_u8()?)?;
    let kind_u8 = c.read_u8()?;
    let flags = c.read_u16::<BigEndian>()?;
    let payload_len = c.read_u32::<BigEndian>()? as usize;

    Ok(Header { dir, kind_u8, flags, payload_len })
}

/// 原始帧（头 + 负载切片）
pub struct RawFrame<'a> {
    pub header: Header,
    pub payload: &'a [u8],
}

/// 仅打包（服务端发送或客户端发送均可复用）
pub fn encode_raw(dir: Direction, kind_u8: u8, payload: &[u8], flags: u16) -> Result<Vec<u8>, FrameError> {
    let mut out = Vec::with_capacity(2 + 1 + 1 + 1 + 2 + 4 + payload.len());
    write_header(&mut out, dir, kind_u8, flags, payload.len())?;
    out.extend_from_slice(payload);
    Ok(out)
}

/// 仅拆包（返回头与 payload 切片）
pub fn decode_raw(bytes: &[u8]) -> Result<RawFrame<'_>, FrameError> {
    let mut c = Cursor::new(bytes);
    let header = read_header(&mut c)?;
    let pos = c.position() as usize;
    if bytes.len() < pos + header.payload_len { return Err(FrameError::Truncated); }
    let payload = &bytes[pos..pos + header.payload_len];
    Ok(RawFrame { header, payload })
}

/// 让“消息类型自己负责序列化”的统一 Trait：
/// - 每个方向各自占用自己的 kind 空间（同一个 kind 值在 C2S/S2C 可含义不同）
/// - 实现后即可直接用默认的 to_frame()/from_frame()
pub trait FrameCodec: Sized {
    /// 该消息所属方向（服务端对 ServerMessage 用 S2C；对 ClientMessage 用 C2S）
    const DIR: Direction;

    /// 返回该消息在其方向下的 kind 值（0..=255）
    fn kind(&self) -> u8;

    /// 该消息本体序列化（不含帧头）
    fn to_bytes(&self) -> Vec<u8>;

    /// 根据 kind + payload 反序列化为本消息类型
    fn from_bytes(kind: u8, payload: &[u8]) -> Result<Self, FrameError>;

    /// 默认：把本消息打成完整帧字节
    fn to_frame(&self) -> Result<Vec<u8>, FrameError> {
        encode_raw(Self::DIR, self.kind(), &self.to_bytes(), 0)
    }

    /// 默认：从完整帧字节解析本消息类型（会校验方向）
    fn from_frame(frame_bytes: &[u8]) -> Result<Self, FrameError> {
        let rf = decode_raw(frame_bytes)?;
        if rf.header.dir != Self::DIR {
            return Err(FrameError::InvalidDirection(rf.header.dir as u8));
        }
        Self::from_bytes(rf.header.kind_u8, rf.payload)
    }
}
