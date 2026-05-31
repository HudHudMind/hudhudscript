//! Internal crypto helpers for e2e operations.

use hudhudscript_bytecode::shared_value::{runtime_error, type_error, SharedResult};
use hudhudscript_bytecode::Value16;
use sha2::{Digest, Sha256, Sha512};

pub fn derive_public_key(private_key: &[u8]) -> Vec<u8> {
    use x25519_dalek::{PublicKey, StaticSecret};
    if private_key.len() != 32 {
        return vec![0u8; 32];
    }
    let mut sk_bytes = [0u8; 32];
    sk_bytes.copy_from_slice(private_key);
    let secret = StaticSecret::from(sk_bytes);
    let public = PublicKey::from(&secret);
    public.as_bytes().to_vec()
}

pub fn compute_shared_secret(private_key: &[u8], public_key: &[u8]) -> Vec<u8> {
    use x25519_dalek::{PublicKey, StaticSecret};
    if private_key.len() != 32 || public_key.len() != 32 {
        return vec![0u8; 32];
    }
    let mut sk_bytes = [0u8; 32];
    sk_bytes.copy_from_slice(private_key);
    let mut pk_bytes = [0u8; 32];
    pk_bytes.copy_from_slice(public_key);
    let secret = StaticSecret::from(sk_bytes);
    let public = PublicKey::from(pk_bytes);
    let shared = secret.diffie_hellman(&public);
    shared.as_bytes().to_vec()
}

pub fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    hmac_sha2::<Sha256>(key, data, 64)
}

pub fn hmac_sha512(key: &[u8], data: &[u8]) -> Vec<u8> {
    hmac_sha2::<Sha512>(key, data, 128)
}

fn hmac_sha2<D: Digest + Clone>(key: &[u8], data: &[u8], block_size: usize) -> Vec<u8> {
    let key = if key.len() > block_size {
        let mut hasher = D::new();
        hasher.update(key);
        hasher.finalize().to_vec()
    } else {
        key.to_vec()
    };

    let mut padded_key = key.clone();
    padded_key.resize(block_size, 0x00);

    let mut i_key_pad = vec![0u8; block_size];
    let mut o_key_pad = vec![0u8; block_size];
    for i in 0..block_size {
        i_key_pad[i] = padded_key[i] ^ 0x36;
        o_key_pad[i] = padded_key[i] ^ 0x5c;
    }

    let mut inner_hasher = D::new();
    inner_hasher.update(&i_key_pad);
    inner_hasher.update(data);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = D::new();
    outer_hasher.update(&o_key_pad);
    outer_hasher.update(&inner_hash);
    outer_hasher.finalize().to_vec()
}

pub fn generate_keystream(key: &[u8], nonce: &[u8], len: usize) -> Vec<u8> {
    let mut stream = Vec::with_capacity(len);
    let mut counter: u32 = 0;
    while stream.len() < len {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(nonce);
        hasher.update(counter.to_le_bytes());
        let block = hasher.finalize();
        stream.extend_from_slice(&block);
        counter += 1;
    }
    stream.truncate(len);
    stream
}

pub fn secure_random_bytes(count: usize) -> SharedResult<Vec<u8>> {
    use std::io::Read;
    let mut buf = vec![0u8; count];

    #[cfg(unix)]
    {
        let mut f = std::fs::File::open("/dev/urandom")
            .map_err(|e| runtime_error(format!("e2e: cannot open /dev/urandom: {}", e)))?;
        f.read_exact(&mut buf)
            .map_err(|e| runtime_error(format!("e2e: failed to read random bytes: {}", e)))?;
    }

    #[cfg(not(unix))]
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut offset = 0;
        while offset < count {
            let mut hasher = DefaultHasher::new();
            offset.hash(&mut hasher);
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .hash(&mut hasher);
            let h = hasher.finish().to_le_bytes();
            let copy_len = std::cmp::min(8, count - offset);
            buf[offset..offset + copy_len].copy_from_slice(&h[..copy_len]);
            offset += copy_len;
        }
    }

    Ok(buf)
}

pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn hex_decode(s: &str, method: &str, param: &str) -> SharedResult<Vec<u8>> {
    hex::decode(s)
        .map_err(|e| runtime_error(format!("{}: invalid hex for {}: {}", method, param, e)))
}

pub fn require_str<'a>(args: &'a [Value16], idx: usize, method: &str) -> SharedResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), method)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            method, idx
        ))),
    }
}
