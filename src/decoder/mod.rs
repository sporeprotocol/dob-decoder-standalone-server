use serde_json::Value;

use crate::{
    client::RpcClient,
    types::{
        ClusterDescriptionField, DOBClusterFormatV0, DOBClusterFormatV1, Dob, Error, Settings,
    },
};

pub(crate) mod helpers;
use helpers::*;

pub struct DOBDecoder {
    rpc: RpcClient,
    settings: Settings,
}

impl DOBDecoder {
    pub fn new(settings: Settings) -> Self {
        Self {
            rpc: RpcClient::new(&settings.ckb_rpc, &settings.ckb_rpc),
            settings,
        }
    }

    pub fn protocol_versions(&self) -> Vec<String> {
        self.settings.protocol_versions.clone()
    }

    pub fn setting(&self) -> &Settings {
        &self.settings
    }

    pub async fn fetch_decode_ingredients(
        &self,
        spore_id: [u8; 32],
    ) -> Result<((Value, String), ClusterDescriptionField), Error> {
        let (content, cluster_id) = fetch_dob_content(&self.rpc, &self.settings, spore_id).await?;
        let dob_metadata = fetch_dob_metadata(&self.rpc, &self.settings, cluster_id).await?;
        Ok((content, dob_metadata))
    }

    // decode DNA under target spore_id
    pub async fn decode_dna(
        &self,
        dna: &str,
        dob_metadata: ClusterDescriptionField,
    ) -> Result<String, Error> {
        let dob = dob_metadata.unbox_dob()?;
        match dob {
            Dob::V0(dob0) => self.decode_dob0_dna(dna, dob0).await,
            Dob::V1(dob1) => self.decode_dob1_dna(dna, dob1).await,
        }
    }

    // decode specificly for objects under DOB/0 protocol
    async fn decode_dob0_dna(&self, dna: &str, dob0: &DOBClusterFormatV0) -> Result<String, Error> {
        let decoder_path = parse_decoder_path(&self.rpc, &dob0.decoder, &self.settings).await?;
        let pattern = match &dob0.pattern {
            Value::String(string) => string.to_owned(),
            pattern => pattern.to_string(),
        };
        let raw_render_result = {
            let (exit_code, outputs) = crate::vm::execute_riscv_binary(
                &decoder_path.to_string_lossy(),
                vec![dna.to_owned().into(), pattern.into()],
                &self.settings,
            )
            .map_err(|_| Error::DecoderExecutionError)?;
            #[cfg(feature = "render_debug")]
            {
                println!("\n-------- DOB/0 DECODE RESULT ({exit_code}) ---------");
                outputs.iter().for_each(|output| println!("{output}"));
                println!("-------- DOB/0 DECODE RESULT END ---------");
            }
            if exit_code != 0 {
                return Err(Error::DecoderExecutionInternalError);
            }
            outputs.first().ok_or(Error::DecoderOutputInvalid)?.clone()
        };
        Ok(raw_render_result)
    }

    // decode specificly for objects under DOB/1 protocol
    async fn decode_dob1_dna(&self, dna: &str, dob1: &DOBClusterFormatV1) -> Result<String, Error> {
        let traits_output = self.decode_dob0_dna(dna, &dob1.traits).await?;
        let image_decoder_path =
            parse_decoder_path(&self.rpc, &dob1.images.decoder, &self.settings).await?;
        let image_pattern = match &dob1.images.pattern {
            Value::String(string) => string.to_owned(),
            pattern => pattern.to_string(),
        };
        let raw_render_result = {
            let (exit_code, outputs) = crate::vm::execute_riscv_binary(
                &image_decoder_path.to_string_lossy(),
                vec![traits_output.into(), image_pattern.into()],
                &self.settings,
            )
            .map_err(|_| Error::DecoderExecutionError)?;
            #[cfg(feature = "render_debug")]
            {
                println!("\n-------- DOB/1 DECODE RESULT ({exit_code}) ---------");
                outputs.iter().for_each(|output| println!("{output}"));
                println!("-------- DOB/1 DECODE RESULT END ---------");
            }
            if exit_code != 0 {
                return Err(Error::DecoderExecutionInternalError);
            }
            outputs.first().ok_or(Error::DecoderOutputInvalid)?.clone()
        };
        Ok(raw_render_result)
    }
}
