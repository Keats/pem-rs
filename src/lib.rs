// Copyright 2016 Jonathan Creekmore
//
// Licensed under the MIT license <LICENSE.md or
// http://opensource.org/licenses/MIT>. This file may not be
// copied, modified, or distributed except according to those terms.

//! This crate provides a parser and encoder for PEM-encoded binary data.
//! PEM-encoded binary data is essentially a beginning and matching end
//! tag that encloses base64-encoded binary data (see:
//! https://en.wikipedia.org/wiki/Privacy-enhanced_Electronic_Mail).
//!
//! This crate's documentation provides a few simple examples along with
//! documentation on the public methods for the crate.
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/pem) and can be used
//! by adding `pem` to your dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! pem = "0.1"
//! ```
//!
//! and this to your crate root:
//!
//! ```rust
//! extern crate pem;
//! ```
//!
//! # Example: parse a single chunk of PEM-encoded text
//!
//! Generally, PEM-encoded files contain a single chunk of PEM-encoded
//! text. Commonly, this is in some sort of a key file or an x.509
//! certificate.
//!
//! ```rust
//!
//! use pem::parse;
//!
//! const SAMPLE: &'static str = "-----BEGIN RSA PRIVATE KEY-----
//! MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
//! dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
//! 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
//! AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
//! DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
//! TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
//! ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
//! -----END RSA PRIVATE KEY-----
//! ";
//!
//!  let pem = parse(SAMPLE).unwrap();
//!  assert_eq!(pem.tag, "RSA PRIVATE KEY");
//! ```
//!
//! # Example: parse a set of PEM-encoded test
//!
//! Sometimes, PEM-encoded files contain multiple chunks of PEM-encoded
//! text. You might see this if you have an x.509 certificate file that
//! also includes intermediate certificates.
//!
//! ```rust
//!
//! use pem::parse_many;
//!
//! const SAMPLE: &'static str = "-----BEGIN INTERMEDIATE CERT-----
//! MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
//! dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
//! 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
//! AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
//! DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
//! TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
//! ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
//! -----END INTERMEDIATE CERT-----
//!
//! -----BEGIN CERTIFICATE-----
//! MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
//! dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
//! 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
//! AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
//! DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
//! TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
//! ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
//! -----END CERTIFICATE-----
//! ";
//!
//!  let pems = parse_many(SAMPLE);
//!  assert_eq!(pems.len(), 2);
//!  assert_eq!(pems[0].tag, "INTERMEDIATE CERT");
//!  assert_eq!(pems[1].tag, "CERTIFICATE");
//! ```

#![deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]

extern crate rustc_serialize;
extern crate regex;

use rustc_serialize::base64::{Config, FromBase64, STANDARD, ToBase64};
use regex::{Captures, Regex};

const PEM_SECTION: &'static str =
    r"(?s)-----BEGIN (?P<begin>.*?)-----\s*(?P<data>.*?)-----END (?P<end>.*?)-----\s*";

/// An error that occurred during parsing Pem-encoded data
#[derive(PartialEq,Clone,Copy,Debug)]
pub enum Error {
    /// PEM-encoded data is not framed correctly
    PemFraming,
    /// Invalid beginning tag
    InvalidBeginTag,
    /// Invalid ending tag
    InvalidEndTag,
    /// Mismatched beginning and ending tags
    MismatchedTags,
    /// Invalid base64-encoded data section
    InvalidData,
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes
    /// sure that clients don't count on exhaustive matching.
    #[doc(hidden)]
    __Nonexhaustive,
}

/// A representation of Pem-encoded data
#[derive(PartialEq,Debug)]
pub struct Pem {
    /// The tag extracted from the Pem-encoded data
    pub tag: String,
    /// The binary contents of the Pem-encoded data
    pub contents: Vec<u8>,
}

fn parse_helper(caps: Captures) -> Result<Pem, Error> {
    // Verify that the begin section exists
    let tag = caps.name("begin").unwrap();
    if tag == "" {
        return Err(Error::InvalidBeginTag);
    }

    // as well as the end section
    let tag_end = caps.name("end").unwrap();
    if tag_end == "" {
        return Err(Error::InvalidEndTag);
    }

    // The beginning and the end sections must match
    if tag != tag_end {
        return Err(Error::MismatchedTags);
    }

    // If they did, then we can grab the data section
    let data = caps.name("data").unwrap();

    // Replace whitespace
    let data = data.replace("\n", "").replace(" ", "");

    // And decode it from Base64 into a vector of u8
    let contents = match data.from_base64() {
        Ok(c) => c,
        Err(_) => {
            return Err(Error::InvalidData);
        }
    };

    Ok(Pem {
        tag: tag.to_owned(),
        contents: contents,
    })
}

/// Parses a single Pem-encoded data from a string.
pub fn parse(input: &str) -> Result<Pem, Error> {
    let re = Regex::new(PEM_SECTION).unwrap();

    match re.captures(input) {
        Some(caps) => parse_helper(caps),
        None => Err(Error::PemFraming),
    }
}

/// Parses a set of Pem-encoded data from a string.
pub fn parse_many(input: &str) -> Vec<Pem> {
    // Create the PEM section regex
    let re = Regex::new(PEM_SECTION).unwrap();

    // Each time our regex matches a PEM section, we need to decode it.
    re.captures_iter(input)
      .filter_map(|caps| parse_helper(caps).ok())
      .collect()
}

/// Encode a Pem struct into a Pem-encoded data string
pub fn encode(pem: &Pem) -> String {
    let mut output = String::new();

    let contents;

    if pem.contents.is_empty() {
        contents = String::from("");
    } else {
        contents = pem.contents.to_base64(Config { line_length: Some(62), ..STANDARD });
    }

    output.push_str(&format!("-----BEGIN {}-----\r\n", pem.tag));
    output.push_str(&format!("{}\r\n", contents));
    output.push_str(&format!("-----END {}-----\r\n", pem.tag));

    output
}

/// Encode multiple Pem structs into a set of Pem-encoded data strings
pub fn encode_many(pems: &[Pem]) -> String {
    pems.iter().map(encode).collect::<Vec<String>>().join("\r\n")
}

#[cfg(test)]
mod test {
    const SAMPLE: &'static str = "-----BEGIN RSA PRIVATE KEY-----\r
MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc\r
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO\r
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei\r
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un\r
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT\r
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh\r
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ\r
-----END RSA PRIVATE KEY-----\r
\r
-----BEGIN RSA PUBLIC KEY-----\r
MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo\r
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0\r
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI\r
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk\r
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6\r
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g\r
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg\r
-----END RSA PUBLIC KEY-----\r
";

    #[test]
    fn parse_works() {
        let pem = super::parse(SAMPLE).unwrap();
        assert_eq!(pem.tag, "RSA PRIVATE KEY");
    }

    #[test]
    fn parse_invalid_framing() {
        let input = "--BEGIN data-----
        -----END data-----";
        match super::parse(&input) {
            Ok(_) => assert!(false),
            Err(code) => assert_eq!(code, super::Error::PemFraming),
        }
    }

    #[test]
    fn parse_invalid_begin() {
        let input = "-----BEGIN -----
MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg
-----END RSA PUBLIC KEY-----";
        match super::parse(&input) {
            Ok(_) => assert!(false),
            Err(code) => assert_eq!(code, super::Error::InvalidBeginTag),
        }
    }

    #[test]
    fn parse_invalid_end() {
        let input = "-----BEGIN DATA-----
MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg
-----END -----";
        match super::parse(&input) {
            Ok(_) => assert!(false),
            Err(code) => assert_eq!(code, super::Error::InvalidEndTag),
        }
    }

    #[test]
    fn parse_invalid_data() {
        let input = "-----BEGIN DATA-----
MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oY?
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg
-----END DATA-----";
        match super::parse(&input) {
            Ok(_) => assert!(false),
            Err(code) => assert_eq!(code, super::Error::InvalidData),
        }
    }

    #[test]
    fn parse_empty_data() {
        let input = "-----BEGIN DATA-----
-----END DATA-----";
        let pem = super::parse(&input).unwrap();
        assert_eq!(pem.contents.len(), 0);
    }

    #[test]
    fn parse_many_works() {
        let pems = super::parse_many(SAMPLE);
        assert_eq!(pems.len(), 2);
        assert_eq!(pems[0].tag, "RSA PRIVATE KEY");
        assert_eq!(pems[1].tag, "RSA PUBLIC KEY");
    }

    #[test]
    fn encode_empty_contents() {
        let pem = super::Pem {
            tag: String::from("FOO"),
            contents: vec![],
        };
        let encoded = super::encode(&pem);
        assert!(encoded != "");

        let pem_out = super::parse(&encoded).unwrap();
        assert_eq!(&pem, &pem_out);
    }

    #[test]
    fn encode_contents() {
        let pem = super::Pem {
            tag: String::from("FOO"),
            contents: vec![1, 2, 3, 4],
        };
        let encoded = super::encode(&pem);
        assert!(encoded != "");

        let pem_out = super::parse(&encoded).unwrap();
        assert_eq!(&pem, &pem_out);
    }

    #[test]
    fn encode_many() {
        let pems = super::parse_many(SAMPLE);
        let encoded = super::encode_many(&pems);

        assert_eq!(SAMPLE, encoded);
    }
}