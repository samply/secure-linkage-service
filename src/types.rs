
use anyhow::{bail, Result};
use base64::{prelude::BASE64_STANDARD, Engine as _};
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer,
};

use crate::Role;

#[derive(Debug, Clone)]
pub struct BooleanShare {
    num_shares: usize,
    shares: Vec<Vec<u8>>,
}

impl BooleanShare {
    fn reconstruct_string(&self) -> Result<String> {
        if self.num_shares != self.shares.len() {
            bail!("Cannot reconstruct: Not all shares set");
        }
        let result: String = {
            if self.num_shares == 1 {
                String::from_utf8(self.shares.first().unwrap().to_owned())?
            } else {
                let mut temp = self.shares.first().unwrap().clone();
                for i in 1..self.num_shares {
                    temp.iter_mut()
                        .zip(&self.shares[i])
                        .for_each(|(a, v)| *a = *a ^ v);
                }
                String::from_utf8(temp)?
            }
        };
        Ok(result)
    }

    fn reconstruct_bool(&self) -> Result<bool> {
        if self.num_shares != self.shares.len() {
            bail!("Cannot reconstruct: Not all shares set");
        }
        let result: bool = {
            if self.num_shares == 1 {
                self.shares
                    .first()
                    .unwrap()
                    .iter()
                    .fold(false, |accumulator, byte| accumulator | (*byte == 0))
            } else {
                let mut temp = self.shares.first().unwrap().clone();
                for i in 1..self.num_shares {
                    temp.iter_mut()
                        .zip(&self.shares[i])
                        .for_each(|(a, v)| *a = *a ^ v);
                }
                temp.iter()
                    .fold(false, |accumulator, byte| accumulator | (*byte != 0))
            }
        };
        Ok(result)
    }
    fn reconstruct_usize(&self) -> Result<usize> {
        if self.num_shares != self.shares.len() {
            bail!("Cannot reconstruct: Not all shares set");
        }
        let mut vec_result: Vec<u8> = Vec::new();
        if self.num_shares == 1 {
            vec_result = self.shares.first().unwrap().to_owned();
        } else {
            let mut temp = self.shares.first().unwrap().clone();
            for i in 1..self.num_shares {
                temp.iter_mut()
                    .zip(&self.shares[i])
                    .for_each(|(a, v)| *a = *a ^ v);
            }
            temp.reverse();
            vec_result = temp;
        }
        // usize is 64bit=8bytes in length
        if vec_result.len() > 8 {
            bail!("Reconstructed usize too long: {} bytes", vec_result.len());
        }
        // Fill up to 64bit len, will do nothing if already 8 bytes
        vec_result.reserve(8-vec_result.len());
        for i in vec_result.len()..8 {
            vec_result.push(0);
        }
        // Convert Vec<u8> to usize
        Ok(usize::from_le_bytes(vec_result.try_into().unwrap()))
    }
}

#[derive(Debug, Clone)]
pub struct Computation {
    client_set: bool,
    server_set: bool,
    match_bit: BooleanShare,
    tmatch_bit: BooleanShare,
    best_index: BooleanShare,
    server_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct CompleteComputation {
    match_bit: bool,
    tmatch_bit: bool,
    best_id: String,
}

impl Computation {
    fn new()-> Self {
        Computation { client_set: false, server_set: false, match_bit: BooleanShare { num_shares: 2, shares: Vec::with_capacity(2) }, tmatch_bit: BooleanShare { num_shares: 2, shares: Vec::with_capacity(2) }, best_index: BooleanShare { num_shares: 2, shares: Vec::with_capacity(2) }, server_ids: None }
    }
    fn add_result(&mut self, share: SMPCResult) -> Result<()> {
        if (self.client_set && share.role == Role::Client)
            || (self.server_set && share.role == Role::Server)
        {
            bail!("Share for the role {:?} already set!", share.role);
        } else {
            let index_bytes = BASE64_STANDARD.decode(share.result.best_idx);
            let index_bytes = index_bytes?;
            self.best_index.shares.push(index_bytes);
            dbg!(&self);
            let match_share = vec![share.result.match_bit as u8];
            self.match_bit.shares.push(match_share);
            let tentative_share = vec![share.result.tentative_match as u8];
            self.tmatch_bit.shares.push(tentative_share);

            if share.role == Role::Server {
                self.server_ids = share.ids;
                self.server_set = true;
            } else {
                self.client_set = true;
            }
            Ok(())
        }
    }
    fn reconstruct_result(&self) -> Result<CompleteComputation> {
        if self.client_set & self.server_set == false {
            bail!("Not all shares set!");
        } else 
        {
            let id = self.best_index.reconstruct_usize()?;
            Ok(CompleteComputation { match_bit: self.match_bit.reconstruct_bool()?, tmatch_bit: self.tmatch_bit.reconstruct_bool()?, best_id: self.server_ids.as_ref().unwrap()[id].to_string() })
        }


    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ShareResult {
    #[serde(rename = "match")]
    #[serde(deserialize_with = "bool_from_int")]
    match_bit: bool,
    #[serde(rename = "tentativeMatch")]
    #[serde(deserialize_with = "bool_from_int")]
    tentative_match: bool,
    #[serde(rename = "bestIndex")]
    best_idx: String,
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SMPCResult {
    role: Role,
    result: ShareResult,
    ids: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn generate_data() -> (String, String) {
        let client_result: String = r#"
{
  "role": "client",
  "result": {
    "match": 0,
    "tentativeMatch": 1,
    "bestIndex": "N6Q="
  }
}
    "#
        .into();
        let server_result: String = r#"
{
  "role": "server",
  "result": {
    "match": 1,
    "tentativeMatch": 1,
    "bestIndex": "N6c="
  },
  "ids": [
    "PseudonymA",
    "PseudonymB",
    "PseudonymC",
    "PseudonymD",
    "PseudonymE"
  ]
}
    "#
        .into();
        (client_result, server_result)
    }
    #[test]
    fn deserialize_mpc_result() {
        let (client_result, server_result) = generate_data();
        let client: SMPCResult =
            serde_json::from_str(&client_result).expect("Client not deserializable");
        let server: SMPCResult =
            serde_json::from_str(&server_result).expect("Server not deserializable");
    }
    #[test]
    fn reconstruct_result() {
        let (client_result, server_result) = generate_data();
        let client: SMPCResult =
            serde_json::from_str(&client_result).expect("Client not deserializable");
        let server: SMPCResult =
            serde_json::from_str(&server_result).expect("Server not deserializable");
        let mut computation = Computation::new();
        computation.add_result(client).unwrap();
        computation.add_result(server).unwrap();
        let result = computation.reconstruct_result().unwrap();
        assert_eq!(result.match_bit, true);
        assert_eq!(result.tmatch_bit, false);
        assert_eq!(result.best_id, "PseudonymD");
    }
}
