use ckb_types::H256;
use serde_json::Value;

use crate::{
    client::RPC,
    types::{
        ClusterDescriptionField, DOBClusterFormatV0, DOBClusterFormatV1, Dob, Error, Settings,
        StandardDOBOutput,
    },
};

pub(crate) mod helpers;
use helpers::*;

pub struct DOBDecoder<T: RPC + 'static> {
    rpc: T,
    settings: Settings,
}

impl<T: RPC> DOBDecoder<T> {
    pub fn new(rpc: T, settings: Settings) -> Self {
        Self { rpc, settings }
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
    ) -> Result<((Value, String), ClusterDescriptionField, H256), Error> {
        let (content, cluster_id, type_hash) =
            fetch_dob_content(&self.rpc, &self.settings, spore_id).await?;
        let dob_metadata = fetch_dob_metadata(&self.rpc, &self.settings, cluster_id).await?;
        Ok((content, dob_metadata, type_hash))
    }

    // decode DNA under target spore_id
    pub async fn decode_dna(
        &self,
        dna: &str,
        dob_metadata: ClusterDescriptionField,
        spore_type_hash: H256,
    ) -> Result<String, Error> {
        let dob = dob_metadata.unbox_dob()?;
        match dob {
            Dob::V0(dob0) => self.decode_dob0_dna(dna, dob0, spore_type_hash).await,
            Dob::V1(dob1) => self.decode_dob1_dna(dna, dob1, spore_type_hash).await,
        }
    }

    // decode specificly for objects under DOB/0 protocol
    async fn decode_dob0_dna(
        &self,
        dna: &str,
        dob0: &DOBClusterFormatV0,
        spore_type_hash: H256,
    ) -> Result<String, Error> {
        let decoder_path = parse_decoder_path(&self.rpc, &dob0.decoder, &self.settings).await?;
        let pattern = match &dob0.pattern {
            Value::String(string) => string.to_owned(),
            pattern => pattern.to_string(),
        };
        let raw_render_result = {
            let (exit_code, outputs) = crate::vm::execute_riscv_binary(
                &decoder_path.to_string_lossy(),
                vec![dna.to_owned().into(), pattern.into()],
                spore_type_hash,
                self.rpc.clone(),
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
    async fn decode_dob1_dna(
        &self,
        dna: &str,
        dob1: &DOBClusterFormatV1,
        spore_type_hash: H256,
    ) -> Result<String, Error> {
        let mut output = Option::<Vec<StandardDOBOutput>>::None;
        for (i, value) in dob1.decoders.iter().enumerate() {
            let decoder_path =
                parse_decoder_path(&self.rpc, &value.decoder, &self.settings).await?;
            let pattern = match &value.pattern {
                Value::String(string) => string.to_owned(),
                pattern => pattern.to_string(),
            };
            let raw_render_result = {
                let args = if let Some(previous_output) = &output {
                    vec![
                        dna.to_owned().into(),
                        pattern.into(),
                        serde_json::to_string(previous_output)
                            .expect("parsed_dna")
                            .into(),
                    ]
                } else {
                    vec![dna.to_owned().into(), pattern.into()]
                };
                let (exit_code, outputs) = crate::vm::execute_riscv_binary(
                    &decoder_path.to_string_lossy(),
                    args,
                    spore_type_hash.clone(),
                    self.rpc.clone(),
                    &self.settings,
                )
                .map_err(|_| Error::DecoderExecutionError)?;
                #[cfg(feature = "render_debug")]
                {
                    println!("\n-------- DOB/1 DECODE RESULT ({i} => {exit_code}) ---------");
                    outputs.iter().for_each(|output| println!("{output}"));
                    println!("-------- DOB/1 DECODE RESULT END ---------");
                }
                if exit_code != 0 {
                    return Err(Error::DecoderExecutionInternalError);
                }
                outputs.first().ok_or(Error::DecoderOutputInvalid)?.clone()
            };
            output = Some(
                serde_json::from_str(&raw_render_result)
                    .map_err(|_| Error::DecoderOutputInvalid)?,
            );
        }
        let Some(output) = output else {
            return Err(Error::DecoderChainIsEmpty);
        };
        Ok(serde_json::to_string(&output).unwrap())
    }
}
