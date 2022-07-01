// Copyright (c) 2022 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0

use flate2;
use serde::Deserialize;
use std::fmt;
use std::io;
use zstd;

/// Represents the layer compression algorithm type,
/// and allows to decompress corresponding compressed data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Compression {
    Uncompressed,
    Gzip,
    Zstd,
}

impl Default for Compression {
    fn default() -> Compression {
        Compression::Gzip
    }
}

impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match self {
            Compression::Uncompressed => "uncompressed",
            Compression::Gzip => "gzip",
            Compression::Zstd => "zstd",
        };

        write!(f, "{}", output)
    }
}

impl Compression {
    /// Decompress input data from one `Read` and output data to one `Write`.
    /// Uncompressed data are not supported and an error will be returned.
    pub fn decompress<R, W>(&self, input: R, output: &mut W) -> std::io::Result<()>
    where
        R: io::Read,
        W: io::Write,
    {
        match *self {
            Self::Gzip => gzip_decode(input, output),
            Self::Zstd => zstd_decode(input, output),
            Self::Uncompressed => Err(io::Error::new(
                io::ErrorKind::Other,
                "uncompressed input data".to_string(),
            )),
        }
    }
}

// Decompress a gzip encoded data with flate2 crate.
fn gzip_decode<R, W>(input: R, output: &mut W) -> std::io::Result<()>
where
    R: io::Read,
    W: io::Write,
{
    let mut decoder = flate2::read::GzDecoder::new(input);
    io::copy(&mut decoder, output)?;
    Ok(())
}

// Decompress a zstd encoded data with zstd crate.
fn zstd_decode<R, W>(input: R, output: &mut W) -> std::io::Result<()>
where
    R: io::Read,
    W: io::Write,
{
    let mut decoder = zstd::Decoder::new(input)?;
    io::copy(&mut decoder, output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use std::io::Write;

    #[test]
    fn test_uncompressed_decode() {
        let bytes = Vec::new();
        let mut output = Vec::new();
        let compression = Compression::Uncompressed;
        assert!(compression
            .decompress(bytes.as_slice(), &mut output)
            .is_err());
    }

    #[test]
    fn test_gzip_decode() {
        let data: Vec<u8> = b"This is some text!".to_vec();

        let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&data).unwrap();
        let bytes = encoder.finish().unwrap();

        let mut output = Vec::new();

        let compression = Compression::Uncompressed;
        assert!(compression
            .decompress(bytes.as_slice(), &mut output)
            .is_err());

        let compression = Compression::default();
        assert!(compression
            .decompress(bytes.as_slice(), &mut output)
            .is_ok());
        assert_eq!(data, output);
    }

    #[test]
    fn test_zstd_decode() {
        let data: Vec<u8> = b"This is some text!".to_vec();
        let level = 1;

        let bytes = zstd::encode_all(&data[..], level).unwrap();

        let mut output = Vec::new();
        let compression = Compression::Zstd;
        assert!(compression
            .decompress(bytes.as_slice(), &mut output)
            .is_ok());
        assert_eq!(data, output);
    }
}
