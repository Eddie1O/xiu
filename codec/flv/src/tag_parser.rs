use super::define;
use super::define::TagType;
use super::errors::TagParseError;
use byteorder::BigEndian;
use bytes::BytesMut;
use netio::bytes_reader::BytesReader;

#[derive(Clone)]
pub struct Tag {
    /*
        SoundFormat: UB[4]
        0 = Linear PCM, platform endian
        1 = ADPCM
        2 = MP3
        3 = Linear PCM, little endian
        4 = Nellymoser 16-kHz mono
        5 = Nellymoser 8-kHz mono
        6 = Nellymoser
        7 = G.711 A-law logarithmic PCM
        8 = G.711 mu-law logarithmic PCM
        9 = reserved
        10 = AAC
        11 = Speex
        14 = MP3 8-Khz
        15 = Device-specific sound
        Formats 7, 8, 14, and 15 are reserved for internal use
        AAC is supported in Flash Player 9,0,115,0 and higher.
        Speex is supported in Flash Player 10 and higher.
    */
    pub sound_format: u8,
    /*
        SoundRate: UB[2]
        Sampling rate
        0 = 5.5-kHz For AAC: always 3
        1 = 11-kHz
        2 = 22-kHz
        3 = 44-kHz
    */
    pub sound_rate: u8,
    /*
        SoundSize: UB[1]
        0 = snd8Bit
        1 = snd16Bit
        Size of each sample.
        This parameter only pertains to uncompressed formats.
        Compressed formats always decode to 16 bits internally
    */
    pub sound_size: u8,
    /*
        SoundType: UB[1]
        0 = sndMono
        1 = sndStereo
        Mono or stereo sound For Nellymoser: always 0
        For AAC: always 1
    */
    pub sound_type: u8,

    /*
        0: AAC sequence header
        1: AAC raw
    */
    aac_packet_type: u8,

    /*
        1: keyframe (for AVC, a seekable frame)
        2: inter frame (for AVC, a non- seekable frame)
        3: disposable inter frame (H.263 only)
        4: generated keyframe (reserved for server use only)
        5: video info/command frame
    */
    pub frame_type: u8,
    /*
        1: JPEG (currently unused)
        2: Sorenson H.263
        3: Screen video
        4: On2 VP6
        5: On2 VP6 with alpha channel
        6: Screen video version 2
        7: AVC
    */
    pub codec_id: u8,
    /*
        0: AVC sequence header
        1: AVC NALU
        2: AVC end of sequence (lower level NALU sequence ender is not required or supported)
    */
    pub avc_packet_type: u8,
    pub composition_time: u32,
}

impl Tag {
    pub fn defalut() -> Self {
        Tag {
            sound_format: 0,
            sound_rate: 0,
            sound_size: 0,
            sound_type: 0,
            aac_packet_type: 0,
            frame_type: 0,
            codec_id: 0,
            avc_packet_type: 0,
            composition_time: 0,
        }
    }
}

pub struct TagParser {
    tag_type: TagType,
    bytes_reader: BytesReader,
    tag: Tag,
}

impl TagParser {
    pub fn new(data: BytesMut, tag_type: TagType) -> Self {
        Self {
            tag_type,
            bytes_reader: BytesReader::new(data),
            tag: Tag::defalut(),
        }
    }
    pub fn parse(&mut self) -> Result<Tag, TagParseError> {
        match self.tag_type {
            TagType::AUDIO => return self.parse_audio_tag_header(),
            TagType::VIDEO => return self.parse_video_tag_header(),
        }
    }

    fn parse_audio_tag_header(&mut self) -> Result<Tag, TagParseError> {
        let flags = self.bytes_reader.read_u8()?;

        self.tag.sound_format = flags >> 4;
        self.tag.sound_rate = (flags >> 2) & 0x03;
        self.tag.sound_size = (flags >> 1) & 0x01;
        self.tag.sound_type = flags & 0x01;

        match self.tag.sound_format {
            define::sound_format::AAC => {
                self.tag.aac_packet_type = self.bytes_reader.read_u8()?;
            }
            _ => {}
        }

        return Ok(self.tag.clone());
    }

    fn parse_video_tag_header(&mut self) -> Result<Tag, TagParseError> {
        let flags = self.bytes_reader.read_u8()?;

        self.tag.frame_type = flags >> 4;
        self.tag.codec_id = flags & 0x0f;

        if self.tag.frame_type == define::frame_type::INTER_FRAME
            || self.tag.frame_type == define::frame_type::KEY_FRAME
        {
            self.tag.avc_packet_type = self.bytes_reader.read_u8()?;

            for _ in 0..3 {
                self.tag.composition_time = self.bytes_reader.read_u32::<BigEndian>()?;
                // print!("==time=={}\n",self.tag.composition_time);
                // self.tag.composition_time = self.tag.composition_time << 8 + time as u32;
            }
        }

        return Ok(self.tag.clone());
    }
}