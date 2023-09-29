use russh::{kex, kex::Name};
use russh_keys::key;

// preferred key exchange algorithms
pub const KEX: &[Name; 4] = &[
    kex::DH_G1_SHA1,
    kex::DH_G14_SHA256,
    kex::CURVE25519,
    kex::CURVE25519_PRE_RFC_8731,
];
