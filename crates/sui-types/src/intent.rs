// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use eyre::eyre;
use fastcrypto::encoding::decode_bytes_hex;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use std::str::FromStr;

#[cfg(test)]
#[path = "unit_tests/intent_tests.rs"]
mod intent_tests;

/// The version here is to distinguish between signing different versions of the struct
/// or enum. Serialized output between two different versions of the same struct/enum
/// might accidentally (or maliciously on purpose) match.
#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum IntentVersion {
    V0 = 0,
}

impl From<u8> for IntentVersion {
    fn from(version: u8) -> Self {
        match version {
            0 => Self::V0,
            _ => panic!("Invalid IntentVersion"),
        }
    }
}

/// This enums specifies the application ID. Two intents in two different applications
/// (i.e., Narwhal, Sui, Ethereum etc) should never collide, so that even when a signing
/// key is reused, nobody can take a signature designated for app_1 and present it as a
/// valid signature for an (any) intent in app_2.
#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum AppId {
    Sui = 0,
}

impl From<u8> for AppId {
    fn from(app_id: u8) -> Self {
        match app_id {
            0 => Self::Sui,
            _ => panic!("Invalid AppId"),
        }
    }
}

impl Default for AppId {
    fn default() -> Self {
        Self::Sui
    }
}

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum IntentScope {
    TransactionData = 0,
    TransactionEffects = 1,
    CheckpointSummary = 2,
    PersonalMessage = 3,
}

impl From<u8> for IntentScope {
    fn from(scope: u8) -> Self {
        match scope {
            0 => Self::TransactionData,
            1 => Self::TransactionEffects,
            2 => Self::CheckpointSummary,
            3 => Self::PersonalMessage,
            _ => panic!("Invalid IntentScope"),
        }
    }
}
/// An intent is a compact struct serves as the domain separator for a message that a signature commits to.
/// It consists of three parts: [enum IntentScope] (what the type of the message is), [enum IntentVersion], [enum AppId] (what application that the signature refers to).
/// It is used to construct [struct IntentMessage] that what a signature commits to.
///
/// The serialization of an Intent is a 3-byte array where each field is represented by a byte.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
pub struct Intent {
    pub scope: IntentScope,
    pub version: IntentVersion,
    pub app_id: AppId,
}

impl FromStr for Intent {
    type Err = eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Vec<u8> = decode_bytes_hex(s).map_err(|_| eyre!("Invalid Intent"))?;
        if s.len() != 3 {
            return Err(eyre!("Invalid Intent"));
        }
        Ok(Self {
            scope: s[0].into(),
            version: s[1].into(),
            app_id: s[2].into(),
        })
    }
}

impl Intent {
    pub fn with_app_id(mut self, app_id: AppId) -> Self {
        self.app_id = app_id;
        self
    }

    pub fn with_scope(mut self, scope: IntentScope) -> Self {
        self.scope = scope;
        self
    }
}

impl Default for Intent {
    fn default() -> Self {
        Self {
            version: IntentVersion::V0,
            scope: IntentScope::TransactionData,
            app_id: AppId::Sui,
        }
    }
}

/// Intent Message is a wrapper around a message with its intent. The message can
/// be any type that implements [trait Serialize]. *ALL* signatures in Sui must commits
/// to the intent message, not the message itself. This guarantees any intent
/// message signed in the system cannot collide with another since they are domain
/// separated by intent.
///
/// The serialization of an IntentMessage is compact: it only appends three bytes
/// to the message itself.
#[derive(Debug, PartialEq, Eq, Serialize, Clone, Hash, Deserialize)]
pub struct IntentMessage<T> {
    pub intent: Intent,
    pub value: T,
}

impl<T> IntentMessage<T> {
    pub fn new(intent: Intent, value: T) -> Self {
        Self { intent, value }
    }
}

/// A person message that wraps around a byte array.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct PersonalMessage {
    pub message: Vec<u8>,
}

pub trait SecureIntent: Serialize + private::SealedIntent {}

pub(crate) mod private {
    use super::IntentMessage;

    pub trait SealedIntent {}
    impl<T> SealedIntent for IntentMessage<T> {}
}
