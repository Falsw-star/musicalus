use base64::{prelude::BASE64_STANDARD, Engine};
use hex::ToHex;
use openssl::{rsa::Rsa, symm::{Cipher, Crypter, Mode}};
use rand::RngExt;
use serde_json::{Value, json};

const IV: &[u8] = b"0102030405060708";
const PRESET_KEY: &[u8] = b"0CoJUm6Qyw8W8jud";
const STD_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

const PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDgtQn2JZ34ZC28NWYpAUd98iZ3
7BUrX/aKzmFbt7clFSs6sXqHauqKWqdtLkF2KexO40H1YTX8z2lSgBBOAxLsvaklV
8k4cBFK9snQXE9/DDaFt6Rr7iVZMldczhC0JNgTz+SHXT6CBHuX3e9SdB1Ua44onc
aTWz7OBGLbCiK45wIDAQAB
-----END PUBLIC KEY-----"#;

const AES_KEY: &[u8] = b"e82ckenh8dichen8";
// const ID_MAGIC: &str = "3go8&$8*3*3h0k(2)2";


pub struct ApiCrypto;

impl ApiCrypto {
    pub fn aes_encrypt(data: &[u8], key: &[u8], iv: Option<&[u8]>, cipher: Cipher) -> anyhow::Result<Vec<u8>> {
        let mut crypter = Crypter::new(cipher, Mode::Encrypt, key, iv)?;
        let mut output = vec![0; data.len() + cipher.block_size()];
        let mut count = crypter.update(&data, &mut output)?;
        count += crypter.finalize(&mut output[count..])?;
        output.truncate(count);
        Ok(output)
    }

    pub fn aes_cbc_encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> anyhow::Result<Vec<u8>> {
        ApiCrypto::aes_encrypt(data, key, Some(iv), Cipher::aes_128_cbc())
    }

    pub fn aes_ecb_encrypt(data: &[u8], key: &[u8]) -> anyhow::Result<Vec<u8>> {
        ApiCrypto::aes_encrypt(data, key, None, Cipher::aes_128_ecb())
    }

    pub fn new_len16_rand() -> (Vec<u8>, Vec<u8>) {
        let mut rng = rand::rng();
        let mut rand_bytes = Vec::with_capacity(16);

        for _ in 0..16 {
            let index = rng.random_range(0..STD_CHARS.len());
            rand_bytes.push(STD_CHARS[index]);
        }

        let rand_bytes_rev = rand_bytes.clone().into_iter().rev().collect::<Vec<u8>>();
        (rand_bytes, rand_bytes_rev)
    }

    pub fn rsa_encrypt_no_padding(key: &[u8]) -> anyhow::Result<Vec<u8>> {
        let rsa = Rsa::public_key_from_pem(PUBLIC_KEY_PEM.as_bytes())?;

        let mut padded_key = vec![0; 128 - key.len()];
        padded_key.extend_from_slice(key);

        let big_num = openssl::bn::BigNum::from_slice(&padded_key)?;
        let e = rsa.e();
        let n = rsa.n();
        let mut ctx = openssl::bn::BigNumContext::new()?;
        let mut result = openssl::bn::BigNum::new()?;

        result.mod_exp(&big_num, &e, &n, &mut ctx)?;

        let encrypted = result.to_vec();
        let mut result_bytes = vec![0u8; 128];
        let start_pos = 128usize.saturating_sub(encrypted.len());
        result_bytes[start_pos..].copy_from_slice(&encrypted);
        Ok(result_bytes)
    }
}

pub fn make_weapi_form(ids: Vec<u64>) -> anyhow::Result<Value> { 
    let data = serde_json::to_string(&json!({
        "c": serde_json::to_string(&Value::Array(
            ids.iter().map(|id| json!({"id": id}))
            .collect::<Vec<Value>>()
        ))?,
        "ids": serde_json::to_string(&Value::Array(
            ids.iter().map(|id| json!(id))
            .collect::<Vec<Value>>()
        ))?,
    }))?;

    let (key, key_rev) = ApiCrypto::new_len16_rand();

    let first_encrypted = ApiCrypto::aes_cbc_encrypt(data.as_bytes(), PRESET_KEY, IV);
    let first_base64 = BASE64_STANDARD.encode(first_encrypted?);

    let second_encrypted = ApiCrypto::aes_cbc_encrypt(first_base64.as_bytes(), &key_rev, IV);
    let params = BASE64_STANDARD.encode(second_encrypted?);

    let encrypted_key = ApiCrypto::rsa_encrypt_no_padding(&key)?;
    let encrypted_key_hex = hex::encode(encrypted_key);

    Ok(json!({
        "params": params,
        "encSecKey": encrypted_key_hex
    }))
}

pub fn make_eapi_header() -> anyhow::Result<String> {
    Ok(serde_json::to_string(&json!({
        "os": "pc",
        "appver": "",
        "osver": "",
        "deviceId": "pyncm!",
        "requestId": rand::random_range(20000000..30000000).to_string()
    }))?)
}

pub fn make_eapi_form(path: String, payload: String) -> anyhow::Result<Value> {
    let text = format!("nobody{path}use{payload}md5forencrypt");
    let digest = openssl::hash::hash(
        openssl::hash::MessageDigest::md5(),
        text.as_bytes()
    )?.encode_hex::<String>();
    let params = format!("{path}-36cd479b6b5-{payload}-36cd479b6b5-{digest}");

    let encrypted_data = ApiCrypto::aes_ecb_encrypt(params.as_bytes(), AES_KEY)?;
    let encrypted_data = hex::encode(&encrypted_data);

    Ok(json!({
        "params": encrypted_data
    }))
}