use bit_vec::BitVec;
use rayon::prelude::*;
use rmp_serde;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

use crate::huffman::{self, Tree};
use Tree::*;

#[derive(serde::Serialize, serde::Deserialize)]
struct CompressedData<T: Eq + Hash> {
    encoder: HashMap<T, BitVec>,
    data: Vec<BitVec>,
}

pub fn compress<'a, T, FreqsF, TokenExtractor, TokensIter>(
    lines: &'a [String],
    get_freqs: FreqsF,
    line_to_tokens: TokenExtractor,
) -> Result<Vec<u8>, Box<dyn std::error::Error>>
where
    T: Clone + Eq + Hash + Send + Sync + Serialize,
    FreqsF: Fn(&'a [String]) -> HashMap<T, u64>,
    TokenExtractor: Fn(&'a str) -> TokensIter + Send + Sync,
    TokensIter: Iterator<Item = T>,
{
    let freqs = get_freqs(lines);
    let tree = huffman::huffman_tree(&freqs);
    let encoder = tree.to_encoder();

    let data = lines
        .par_iter()
        .map(|line| {
            let mut bits = BitVec::new();
            for token in line_to_tokens(line) {
                bits.extend(encoder.get(&token).unwrap().iter());
            }
            bits
        })
        .collect();

    let compressed_data = CompressedData { encoder, data };
    rmp_serde::encode::to_vec(&compressed_data).map_err(|err| err.into())
}

pub fn extract<'a, T, F>(
    data: &'a [u8],
    tokens_to_line: F,
) -> Result<Vec<String>, Box<dyn std::error::Error>>
where
    T: Clone + Eq + Hash + Send + Sync + Deserialize<'a>,
    F: Fn(Vec<T>) -> String + Send + Sync,
{
    let CompressedData { encoder, data }: CompressedData<T> = rmp_serde::decode::from_slice(data)?;

    let decoder = build_decoder(&encoder);
    let lines = data
        .par_iter()
        .map(|line| {
            let mut tokens = Vec::new();
            let mut node = &decoder;

            for bit in line.iter() {
                node = if bit {
                    node.right.as_ref().expect("invalid Huffman bit sequence")
                } else {
                    node.left.as_ref().expect("invalid Huffman bit sequence")
                };

                if let Some(token) = &node.token {
                    tokens.push(token.clone());
                    node = &decoder;
                }
            }
            tokens_to_line(tokens)
        })
        .collect();

    Ok(lines)
}

struct DecoderNode<T> {
    token: Option<T>,
    left: Option<Box<DecoderNode<T>>>,
    right: Option<Box<DecoderNode<T>>>,
}

impl<T> Default for DecoderNode<T> {
    fn default() -> Self {
        DecoderNode {
            token: None,
            left: None,
            right: None,
        }
    }
}

fn build_decoder<T: Clone + Eq + Hash>(encoder: &HashMap<T, BitVec>) -> DecoderNode<T> {
    let mut root = DecoderNode::default();

    for (token, code) in encoder {
        let mut node = &mut root;
        for bit in code.iter() {
            node = if bit {
                node.right.get_or_insert_with(|| Box::new(DecoderNode::default()))
            } else {
                node.left.get_or_insert_with(|| Box::new(DecoderNode::default()))
            };
        }
        node.token = Some(token.clone());
    }

    root
}

impl<T: Eq + Clone + Hash> Tree<T> {
    pub fn to_encoder(&self) -> HashMap<T, BitVec> {
        let mut encoder = HashMap::new();

        let mut stack = vec![(self, BitVec::new())];
        while let Some((node, path)) = stack.pop() {
            match node {
                Leaf { token, .. } => {
                    encoder.insert(token.clone(), path.clone());
                }
                Node { left, right, .. } => {
                    let mut left_path = path.clone();
                    left_path.push(false);
                    stack.push((left, left_path));

                    let mut right_path = path.clone();
                    right_path.push(true);
                    stack.push((right, right_path));
                }
            }
        }

        encoder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::freq::{char_frequencies, word_frequencies};

    #[test]
    fn compress_decompress_test() {
        let lines = vec![
            "hey there! nice to meet you.".to_string(),
            "Serde is a framework for serializing and deserializing Rust data structures"
                .to_string(),
        ];

        let data = compress(&lines, char_frequencies, |line| line.chars()).unwrap();
        let res_lines = extract(&data, |x: Vec<char>| x.into_iter().collect()).unwrap();
        assert_eq!(&lines, &res_lines);

        let data = compress(&lines, word_frequencies, |line| {
            line.split_ascii_whitespace().map(|token| token.to_string())
        })
        .unwrap();
        let res_lines = extract(&data, |x: Vec<String>| x.join(" ")).unwrap();
        assert_eq!(&lines, &res_lines);
    }
}