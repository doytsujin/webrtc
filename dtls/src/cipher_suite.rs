pub mod cipher_suite_tls_ecdhe_ecdsa_with_aes_128_gcm_sha256;
pub mod cipher_suite_tls_ecdhe_ecdsa_with_aes_256_cbc_sha;

use std::fmt;

use async_trait::async_trait;

use super::client_certificate_type::*;
use super::errors::*;
use super::record_layer::*;

use util::Error;

use cipher_suite_tls_ecdhe_ecdsa_with_aes_128_gcm_sha256::*;
use cipher_suite_tls_ecdhe_ecdsa_with_aes_256_cbc_sha::*;

// CipherSuiteID is an ID for our supported CipherSuites
// Supported Cipher Suites
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CipherSuiteID {
    // AES-128-CCM
    TLS_ECDHE_ECDSA_WITH_AES_128_CCM = 0xc0ac,
    TLS_ECDHE_ECDSA_WITH_AES_128_CCM_8 = 0xc0ae,

    // AES-128-GCM-SHA256
    TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 = 0xc02b,
    TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 = 0xc02f,

    // AES-256-CBC-SHA
    TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA = 0xc00a,
    TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA = 0xc014,

    TLS_PSK_WITH_AES_128_CCM = 0xc0a4,
    TLS_PSK_WITH_AES_128_CCM_8 = 0xc0a8,
    TLS_PSK_WITH_AES_128_GCM_SHA256 = 0x00a8,

    Unsupported,
}

impl fmt::Display for CipherSuiteID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_128_CCM => {
                write!(f, "TLS_ECDHE_ECDSA_WITH_AES_128_CCM")
            }
            CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_128_CCM_8 => {
                write!(f, "TLS_ECDHE_ECDSA_WITH_AES_128_CCM_8")
            }
            CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 => {
                write!(f, "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256")
            }
            CipherSuiteID::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 => {
                write!(f, "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256")
            }
            CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA => {
                write!(f, "TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA")
            }
            CipherSuiteID::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA => {
                write!(f, "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA")
            }
            CipherSuiteID::TLS_PSK_WITH_AES_128_CCM => write!(f, "TLS_PSK_WITH_AES_128_CCM"),
            CipherSuiteID::TLS_PSK_WITH_AES_128_CCM_8 => write!(f, "TLS_PSK_WITH_AES_128_CCM_8"),
            CipherSuiteID::TLS_PSK_WITH_AES_128_GCM_SHA256 => {
                write!(f, "TLS_PSK_WITH_AES_128_GCM_SHA256")
            }
            _ => write!(f, "Unsupported CipherSuiteID"),
        }
    }
}

impl From<u16> for CipherSuiteID {
    fn from(val: u16) -> Self {
        match val {
            // AES-128-CCM
            0xc0ac => CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_128_CCM,
            0xc0ae => CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_128_CCM_8,

            // AES-128-GCM-SHA256
            0xc02b => CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
            0xc02f => CipherSuiteID::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,

            // AES-256-CBC-SHA
            0xc00a => CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA,
            0xc014 => CipherSuiteID::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,

            0xc0a4 => CipherSuiteID::TLS_PSK_WITH_AES_128_CCM,
            0xc0a8 => CipherSuiteID::TLS_PSK_WITH_AES_128_CCM_8,
            0x00a8 => CipherSuiteID::TLS_PSK_WITH_AES_128_GCM_SHA256,

            _ => CipherSuiteID::Unsupported,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CipherSuiteHash {
    SHA256,
}

impl CipherSuiteHash {
    pub(crate) fn size(&self) -> usize {
        match *self {
            CipherSuiteHash::SHA256 => 32,
        }
    }
}

#[async_trait]
pub trait CipherSuite {
    fn to_string(&self) -> String;
    fn id(&self) -> CipherSuiteID;
    fn certificate_type(&self) -> ClientCertificateType;
    fn hash_func(&self) -> CipherSuiteHash;
    fn is_psk(&self) -> bool;
    async fn is_initialized(&self) -> bool;

    // Generate the internal encryption state
    async fn init(
        &mut self,
        master_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
        is_client: bool,
    ) -> Result<(), Error>;

    async fn encrypt(&self, pkt: &RecordLayer, raw: &[u8]) -> Result<Vec<u8>, Error>;
    async fn decrypt(&self, input: &[u8]) -> Result<Vec<u8>, Error>;
}

// Taken from https://www.iana.org/assignments/tls-parameters/tls-parameters.xml
// A cipher_suite is a specific combination of key agreement, cipher and MAC
// function.
pub fn cipher_suite_for_id(id: CipherSuiteID) -> Result<Box<dyn CipherSuite>, Error> {
    match id {
        //TODO: complete all cipher suites
        /*CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_128_CCM =>
        return newCipherSuiteTLSEcdheEcdsaWithAes128Ccm()
            CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_128_CCM_8:
        return newCipherSuiteTLSEcdheEcdsaWithAes128Ccm8()
        */
        CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 => {
            Ok(Box::new(CipherSuiteTLSEcdheEcdsaWithAes128GcmSha256::new()))
        }
        /*    CipherSuiteID::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256:
        return &cipherSuiteTLSEcdheRsaWithAes128GcmSha256{}
         */
        CipherSuiteID::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA => {
            Ok(Box::new(CipherSuiteTLSEcdheEcdsaWithAes256CbcSha::new()))
        }
        /*   CipherSuiteID::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA:
        return &cipherSuiteTLSEcdheRsaWithAes256CbcSha{}
            CipherSuiteID::TLS_PSK_WITH_AES_128_CCM:
        return newCipherSuiteTLSPskWithAes128Ccm()
            CipherSuiteID::TLS_PSK_WITH_AES_128_CCM_8:
        return newCipherSuiteTLSPskWithAes128Ccm8()
            CipherSuiteID::TLS_PSK_WITH_AES_128_GCM_SHA256:
        return &cipherSuiteTLSPskWithAes128GcmSha256{}*/
        _ => Err(ERR_INVALID_CIPHER_SUITE.clone()),
    }
}

// CipherSuites we support in order of preference
fn default_cipher_suites() -> Vec<Box<dyn CipherSuite>> {
    vec![
        Box::new(CipherSuiteTLSEcdheEcdsaWithAes128GcmSha256::new()),
        Box::new(CipherSuiteTLSEcdheEcdsaWithAes256CbcSha::new()),
        //TODO: Box::new(CipherSuiteTLSEcdheRsaWithAes128GcmSha256{},
        //TODO: Box::new(CipherSuiteTLSEcdheRsaWithAes256CbcSha{},
    ]
}

fn all_cipher_suites() -> Vec<Box<dyn CipherSuite>> {
    vec![
        //TODO: newCipherSuiteTLSEcdheEcdsaWithAes128Ccm(),
        //TODO: newCipherSuiteTLSEcdheEcdsaWithAes128Ccm8(),
        Box::new(CipherSuiteTLSEcdheEcdsaWithAes128GcmSha256::new()),
        Box::new(CipherSuiteTLSEcdheEcdsaWithAes256CbcSha::new()),
        //TODO: &cipherSuiteTLSEcdheRsaWithAes128GcmSha256{},
        //TODO: &cipherSuiteTLSEcdheRsaWithAes256CbcSha{},
        //TODO: newCipherSuiteTLSPskWithAes128Ccm(),
        //TODO: newCipherSuiteTLSPskWithAes128Ccm8(),
        //TODO: &cipherSuiteTLSPskWithAes128GcmSha256{},
    ]
}

fn cipher_suites_for_ids(ids: &[CipherSuiteID]) -> Result<Vec<Box<dyn CipherSuite>>, Error> {
    let mut cipher_suites = vec![];
    for id in ids {
        cipher_suites.push(cipher_suite_for_id(*id)?);
    }
    Ok(cipher_suites)
}

pub(crate) fn parse_cipher_suites(
    user_selected_suites: &[CipherSuiteID],
    exclude_psk: bool,
    exclude_non_psk: bool,
) -> Result<Vec<Box<dyn CipherSuite>>, Error> {
    let cipher_suites = if user_selected_suites.len() != 0 {
        cipher_suites_for_ids(user_selected_suites)?
    } else {
        default_cipher_suites()
    };

    let filtered_cipher_suites: Vec<Box<dyn CipherSuite>> = cipher_suites
        .into_iter()
        .filter(|c| !((exclude_psk && c.is_psk()) || (exclude_non_psk && !c.is_psk())))
        .collect();

    if filtered_cipher_suites.len() == 0 {
        Err(ERR_NO_AVAILABLE_CIPHER_SUITES.clone())
    } else {
        Ok(filtered_cipher_suites)
    }
}
