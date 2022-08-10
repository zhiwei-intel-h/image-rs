// Copyright (c) 2022 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use ocicrypt_rs::config::CryptoConfig;
use ocicrypt_rs::encryption::decrypt_layer;
use ocicrypt_rs::helpers::create_decrypt_config;
use ocicrypt_rs::spec::{
    MEDIA_TYPE_LAYER_ENC, MEDIA_TYPE_LAYER_GZIP_ENC, MEDIA_TYPE_LAYER_NON_DISTRIBUTABLE_ENC,
    MEDIA_TYPE_LAYER_NON_DISTRIBUTABLE_GZIP_ENC,
};

use oci_distribution::manifest;
use oci_distribution::manifest::OciDescriptor;

use std::io::Read;

#[derive(Default, Clone)]
pub struct Decryptor {
    /// The layer original media type before encryption.
    pub media_type: String,

    /// Whether layer is encrypted.
    encrypted: bool,
}

impl Decryptor {
    /// Construct Decryptor from media_type.
    pub fn from_media_type(media_type: &str) -> Self {
        let (media_type, encrypted) = match media_type {
            MEDIA_TYPE_LAYER_ENC | MEDIA_TYPE_LAYER_NON_DISTRIBUTABLE_ENC => {
                (manifest::IMAGE_LAYER_MEDIA_TYPE.to_string(), true)
            }
            MEDIA_TYPE_LAYER_GZIP_ENC | MEDIA_TYPE_LAYER_NON_DISTRIBUTABLE_GZIP_ENC => {
                (manifest::IMAGE_LAYER_GZIP_MEDIA_TYPE.to_string(), true)
            }
            _ => ("".to_string(), false),
        };

        Decryptor {
            media_type,
            encrypted,
        }
    }

    /// Check whether media_type is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.encrypted
    }

    /// get_plaintext_layer descrypts encrypted_layer data and return the
    /// plaintext_layer data. descriptor and decrypt_config are required for
    /// layer data decryption process.
    ///
    /// * `decrypt_config` - decryption key info in following format:\
    ///           - \<filename> \
    ///           - \<filename>:file=\<passwordfile> \
    ///           - \<filename>:pass=\<password> \
    ///           - \<filename>:fd=\<filedescriptor> \
    ///           - \<filename>:\<password> \
    ///           - provider:<cmd/gprc>
    pub async fn get_plaintext_layer(
        &self,
        descriptor: &OciDescriptor,
        encrypted_layer: Vec<u8>,
        decrypt_config: &str,
    ) -> Result<Vec<u8>> {
        if !self.is_encrypted() {
            return Err(anyhow!("unencrypted media type: {}", self.media_type));
        }

        if decrypt_config.is_empty() {
            return Err(anyhow!("decrypt_config is empty"));
        }

        let cc = create_decrypt_config(vec![decrypt_config.to_string()], vec![])?;
        let descript = descriptor.clone();

        // ocicrypt-rs keyprovider module will create a new runtime to talk with
        // attestation agent, to avoid startup a runtime within a runtime, we
        // spawn a new thread here.
        let handler = tokio::task::spawn_blocking(move || {
            decrypt_layer_data(&encrypted_layer, &descript, &cc)
        });

        if let Ok(decrypted_data) = handler.await? {
            Ok(decrypted_data)
        } else {
            Err(anyhow!("decrypt failed!"))
        }
    }
}

fn decrypt_layer_data(
    encrypted_layer: &[u8],
    descriptor: &OciDescriptor,
    crypto_config: &CryptoConfig,
) -> Result<Vec<u8>> {
    if let Some(decrypt_config) = &crypto_config.decrypt_config {
        let (layer_decryptor, _dec_digest) =
            decrypt_layer(decrypt_config, encrypted_layer, descriptor, false)?;
        let mut plaintext_data: Vec<u8> = Vec::new();
        let mut decryptor = layer_decryptor.ok_or_else(|| anyhow!("missing layer decryptor"))?;

        decryptor.read_to_end(&mut plaintext_data)?;

        Ok(plaintext_data)
    } else {
        Err(anyhow!("no decrypt config available"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oci_distribution::manifest::{OciDescriptor};
    use std::collections::HashMap;
    use futures::executor::block_on;
    use sha2::Digest;
    #[tokio::test]
    async fn test_get_plaintext_layer() {
        // set env
        let config_dir = std::env!("CARGO_MANIFEST_DIR");
        let keyprovider_config =
            format!("{}/{}", config_dir, "test_data/ocicrypt_keyprovider_native.conf");
        std::env::set_var("OCICRYPT_KEYPROVIDER_CONFIG", keyprovider_config);

        // layer meta data
        let mut hm = HashMap::new();
        hm.insert(
            "org.opencontainers.image.enc.keys.provider.attestation-agent".to_string(),
            "eyJraWQiOiJudWxsIiwid3JhcHBlZF9kYXRhIjpbMTksMjEzLDk5LDE0OCwxMTYsMjQ1LDIxLDE1MCwyNDIsMTczLDE1MywyMCwxMTQsMTIwLDE4Miw3MCw1MywyNSwxODUsMTU3LDU4LDM2LDE3OSwyMDAsMjMwLDMwLDIwNyw5NCwyMTgsMTY1LDE4LDI0LDI5LDIwMCw0NSw2MywyOSwxODIsMTgzLDE5LDE3NSwxNiwyMDAsMzQsMTMsMTM1LDI0MCwyNDIsMTQ3LDg3LDI0NiwxMDksMTM3LDIzNCwxMjksMjE5LDg5LDExNSwxOTYsMzAsMjEsMzcsMTkyLDI0LDE1Nyw2OSwxMTMsMTU2LDE2MCwxMzgsNzAsMzAsMzksNDMsMTcyLDE2OSwyMiwxNTUsMjUsMjQyLDIwMSw5OCwyNTAsNzYsMTU5LDEyOSwxODcsNSwyMzMsMTYwLDEzMywxOTEsNjQsMTg2LDAsMjE0LDE0NSw2MiwxNzksMTQyLDEzOCwxMjksMTY1LDIwOCwyMDcsMjEyLDUsMTIxLDIzLDQ1LDEwOSwxMDksMTMwLDE1LDE0NywxMzcsMjAyLDQxLDQ2LDM2LDEwMSwxMCwxNzcsMjYsMjUyLDEzOCwyOSwxNzYsMjI1LDEzMCw2NywxNzcsMTc4LDMwLDEwNSw2MiwxNjYsMjAsMCwxNzEsMTA4LDE1LDE4MywyMTgsMTExLDE5MSwxOTIsOTksMTA1LDIyNywxMywxNjQsMTYsMTA0LDI0MSwyMDgsMTY3LDEzNiwyOSw0OSwyNSwxOTIsOTksOTUsNzcsMjI4LDQsMTIwLDE5NSwzNiwyMTcsNjIsMTIxLDIzNyw2MSwyMCwxMDgsODMsMTkxLDE1Myw2MCwzOCwxODMsMTk2LDg0LDE5NiwyOSwxNTgsODcsMTQ4LDExMCwxMzMsMTMzLDExOCwxOTUsMTM2LDE2NSwyMzksMTIwLDQ2LDE4NywxNjgsNDUsMTI3LDI0NCw2MywyMTksMTYsMTldLCJpdiI6WzM5LDIzOCw2MCw0NywyMDUsMTIwLDEwNSwxMTksMjIzLDg5LDQ1LDQ2XSwid3JhcF90eXBlIjoiYWVzLWdjbSJ9".to_string());
        hm.insert(
            "org.opencontainers.image.enc.pubopts".to_string(),
            "eyJjaXBoZXIiOiJBRVNfMjU2X0NUUl9ITUFDX1NIQTI1NiIsImhtYWMiOiI1TXNZSUZoRWdZMGhHNk50MUs2YWZhaUFrRVptV01wcSttbU8xemJkcytFPSIsImNpcGhlcm9wdGlvbnMiOnt9fQ==".to_string());
        let layer_od = OciDescriptor {
            media_type: "application/vnd.oci.image.layer.v1.tar+encrypted".to_string(),
            digest: "sha256:39d2a3d8983573da8c7d9367e515b29148cae3acadc456fa5b4ed78a0e184b4f".to_string(),
            size: 3072,
            annotations: Some(hm),
            ..Default::default()
        };


        // layer data
        let layer_data = std::fs::read(&"test_data/encrypted_layer").unwrap();

        // dc
        let dc = "provider:attestation-agent:sample_kbc::null";
        // decryption
        let decryptor = Decryptor::from_media_type(&layer_od.media_type);
        let plaintext_layer = block_on(decryptor.get_plaintext_layer(&layer_od, layer_data, &dc)).unwrap();
        //println!("{:#?}", plaintext_layer);

        // verify digest
        let digest = format!("{}:{:x}", "sha256", sha2::Sha256::digest(&plaintext_layer.as_slice()));
        let decrypted_digest = "sha256:dfe7577521f0d1ad9f82862f3550e12a615fcb07319265a3d23e96f2f21f62ec".to_string();
        assert_eq!(digest, decrypted_digest);
    }



}
