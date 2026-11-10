//! `Decoder::subscribe`: chunks of a subscribed type are kept whole in `Info::subscribed_chunks`.

use std::io::Cursor;

use png::chunk::ChunkType;
use png::{BitDepth, ColorType, Decoder, DecodingError, Encoder, Limits};

const PRVT: ChunkType = ChunkType(*b"prVt");

/// A 2x2 grey PNG, with `before` written ahead of the image data and `after` behind it.
fn png_with(before: &[(ChunkType, &[u8])], after: &[(ChunkType, &[u8])]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, 2, 2);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    for (type_, data) in before {
        writer.write_chunk(*type_, data).unwrap();
    }
    writer.write_image_data(&[0, 64, 128, 255]).unwrap();
    for (type_, data) in after {
        writer.write_chunk(*type_, data).unwrap();
    }
    writer.finish().unwrap();
    bytes
}

/// The subscribed chunks of a PNG, read to its end.
fn read(
    png: &[u8],
    subscribe: &[ChunkType],
    limits: Limits,
) -> Result<Vec<(ChunkType, Vec<u8>)>, DecodingError> {
    let mut decoder = Decoder::new_with_limits(Cursor::new(png), limits);
    for type_ in subscribe {
        decoder.subscribe(*type_);
    }
    let mut reader = decoder.read_info()?;
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    reader.next_frame(&mut pixels)?;
    reader.finish()?;
    Ok(reader.info().subscribed_chunks.clone())
}

#[test]
fn a_subscribed_chunk_is_kept_whole() {
    let data: Vec<u8> = (0..=255).cycle().take(100_000).collect();
    let png = png_with(&[(PRVT, &data)], &[]);
    assert_eq!(
        read(&png, &[PRVT], Limits::default()).unwrap(),
        vec![(PRVT, data)]
    );
}

#[test]
fn an_unsubscribed_chunk_is_skipped_as_before() {
    let png = png_with(&[(PRVT, b"kept nowhere")], &[]);
    assert_eq!(read(&png, &[], Limits::default()).unwrap(), vec![]);
}

#[test]
fn chunks_after_the_image_data_are_kept_too_in_file_order() {
    let png = png_with(&[(PRVT, b"one")], &[(PRVT, b"two"), (PRVT, b"")]);
    let kept = read(&png, &[PRVT], Limits::default()).unwrap();
    assert_eq!(
        kept,
        vec![
            (PRVT, b"one".to_vec()),
            (PRVT, b"two".to_vec()),
            (PRVT, vec![])
        ]
    );
}

#[test]
fn a_chunk_past_the_limits_is_an_error() {
    let data = vec![7; 1 << 20];
    let png = png_with(&[(PRVT, &data)], &[]);
    let limits = Limits { bytes: 1 << 16 };
    assert!(matches!(
        read(&png, &[PRVT], limits),
        Err(DecodingError::LimitsExceeded)
    ));
}

#[test]
fn a_chunk_with_a_bad_crc_is_not_kept() {
    let mut png = png_with(&[(PRVT, b"tampered")], &[]);
    let at = png.windows(4).position(|w| w == b"prVt").unwrap() + 4;
    png[at] ^= 1;
    assert_eq!(read(&png, &[PRVT], Limits::default()).unwrap(), vec![]);
}

#[test]
fn a_type_the_crate_decodes_is_decoded_as_before() {
    let png = png_with(&[(png::chunk::tEXt, b"Title\0kiku")], &[]);
    let mut decoder = Decoder::new(Cursor::new(&png));
    decoder.subscribe(png::chunk::tEXt);
    let reader = decoder.read_info().unwrap();
    assert!(reader.info().subscribed_chunks.is_empty());
    assert_eq!(reader.info().uncompressed_latin1_text[0].text, "kiku");
}

#[test]
fn an_unknown_critical_chunk_is_still_an_error() {
    let critical = ChunkType(*b"PRVT");
    let png = png_with(&[(critical, b"must be understood")], &[]);
    assert!(read(&png, &[critical], Limits::default()).is_err());
}
