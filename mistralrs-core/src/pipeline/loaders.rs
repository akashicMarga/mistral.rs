use std::str::FromStr;

use anyhow::Result;
use candle_nn::{Activation, VarBuilder};
use mistralrs_lora::{LoraConfig, Ordering};
use pyo3::pyclass;
use serde::Deserialize;

use super::{NormalModel, NormalModelLoader};
use crate::{
    models,
    xlora_models::{self, XLoraConfig},
};

#[pyclass]
#[derive(Clone, Debug)]
/// The architecture to load the normal model as.
pub enum NormalLoaderType {
    Mistral,
    Gemma,
    Mixtral,
    Llama,
    Phi2,
}

impl FromStr for NormalLoaderType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mistral" => Ok(Self::Mistral),
            "gemma" => Ok(Self::Gemma),
            "mixtral" => Ok(Self::Mixtral),
            "llama" => Ok(Self::Llama),
            "phi2" => Ok(Self::Phi2),
            a => Err(format!("Unknown architecture `{a}`")),
        }
    }
}

// ======================== Mistral loader

#[derive(Deserialize)]
pub struct MistralBasicConfig {
    vocab_size: usize,
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: usize,
    hidden_act: Activation,
    max_position_embeddings: usize,
    rms_norm_eps: f64,
    rope_theta: f64,
    sliding_window: Option<usize>,
}

impl MistralBasicConfig {
    fn deserialize(slice: &str, use_flash_attn: bool) -> Result<models::mistral::Config> {
        let basic_config: Self = serde_json::from_str(slice)?;
        Ok(models::mistral::Config {
            vocab_size: basic_config.vocab_size,
            hidden_size: basic_config.hidden_size,
            intermediate_size: basic_config.intermediate_size,
            num_hidden_layers: basic_config.num_hidden_layers,
            num_attention_heads: basic_config.num_attention_heads,
            num_key_value_heads: basic_config.num_key_value_heads,
            hidden_act: basic_config.hidden_act,
            max_position_embeddings: basic_config.max_position_embeddings,
            rms_norm_eps: basic_config.rms_norm_eps,
            rope_theta: basic_config.rope_theta,
            sliding_window: basic_config.sliding_window,
            use_flash_attn,
        })
    }
}

pub struct MistralLoader;

impl NormalModelLoader for MistralLoader {
    fn load(
        &self,
        config: &str,
        use_flash_attn: bool,
        vb: VarBuilder,
    ) -> Result<Box<dyn NormalModel + Send + Sync>> {
        Ok(Box::new(models::mistral::Model::new(
            &MistralBasicConfig::deserialize(config, use_flash_attn)?,
            vb,
            self.is_gptx(),
        )?))
    }
    fn load_xlora(
        &self,
        config: &str,
        use_flash_attn: bool,
        vb: VarBuilder,
        lora_config: &[(String, LoraConfig)],
        xlora_config: Option<XLoraConfig>,
        xlora_ordering: Ordering,
    ) -> Result<Box<dyn NormalModel + Send + Sync>> {
        Ok(Box::new(xlora_models::XLoraMistral::new(
            &MistralBasicConfig::deserialize(config, use_flash_attn)?,
            vb,
            lora_config,
            xlora_config,
            xlora_ordering,
            self.is_gptx(),
        )?))
    }
    fn is_gptx(&self) -> bool {
        true
    }
}

// ======================== Gemma loader

fn default_max_position_embeddings() -> usize {
    4096
}

#[derive(Deserialize)]
pub struct GemmaBasicConfig {
    pub attention_bias: bool,
    pub head_dim: usize,
    // The code gemma configs include both hidden_act and hidden_activation.
    pub hidden_act: Option<Activation>,
    pub hidden_activation: Option<Activation>,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_attention_heads: usize,
    pub num_hidden_layers: usize,
    pub num_key_value_heads: usize,
    pub rms_norm_eps: f64,
    pub rope_theta: f64,
    pub vocab_size: usize,

    #[serde(default = "default_max_position_embeddings")]
    pub max_position_embeddings: usize,
}

impl GemmaBasicConfig {
    fn deserialize(slice: &str, _use_flash_attn: bool) -> Result<models::gemma::Config> {
        let basic_config: Self = serde_json::from_str(slice)?;
        Ok(models::gemma::Config {
            vocab_size: basic_config.vocab_size,
            hidden_size: basic_config.hidden_size,
            intermediate_size: basic_config.intermediate_size,
            num_hidden_layers: basic_config.num_hidden_layers,
            num_attention_heads: basic_config.num_attention_heads,
            num_key_value_heads: basic_config.num_key_value_heads,
            hidden_act: basic_config.hidden_act,
            hidden_activation: basic_config.hidden_activation,
            max_position_embeddings: basic_config.max_position_embeddings,
            rms_norm_eps: basic_config.rms_norm_eps,
            rope_theta: basic_config.rope_theta,
            attention_bias: basic_config.attention_bias,
            head_dim: basic_config.head_dim,
        })
    }
}

pub struct GemmaLoader;

impl NormalModelLoader for GemmaLoader {
    fn load(
        &self,
        config: &str,
        use_flash_attn: bool,
        vb: VarBuilder,
    ) -> Result<Box<dyn NormalModel + Send + Sync>> {
        Ok(Box::new(models::gemma::Model::new(
            &GemmaBasicConfig::deserialize(config, use_flash_attn)?,
            vb,
            self.is_gptx(),
        )?))
    }
    fn load_xlora(
        &self,
        config: &str,
        use_flash_attn: bool,
        vb: VarBuilder,
        lora_config: &[(String, LoraConfig)],
        xlora_config: Option<XLoraConfig>,
        xlora_ordering: Ordering,
    ) -> Result<Box<dyn NormalModel + Send + Sync>> {
        Ok(Box::new(xlora_models::XLoraGemma::new(
            &GemmaBasicConfig::deserialize(config, use_flash_attn)?,
            vb,
            lora_config,
            xlora_config,
            xlora_ordering,
            self.is_gptx(),
        )?))
    }
    fn is_gptx(&self) -> bool {
        true
    }
}

// ======================== Llama loader

#[derive(Deserialize)]
pub struct LlamaBasicConfig {
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub vocab_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: Option<usize>,
    pub rms_norm_eps: f64,
    #[serde(default = "default_rope")]
    pub rope_theta: f32,
}

fn default_rope() -> f32 {
    10_000.0
}

impl LlamaBasicConfig {
    fn deserialize(slice: &str, use_flash_attn: bool) -> Result<models::llama::Config> {
        let basic_config: Self = serde_json::from_str(slice)?;
        Ok(models::llama::Config {
            hidden_size: basic_config.hidden_size,
            intermediate_size: basic_config.intermediate_size,
            vocab_size: basic_config.vocab_size,
            num_hidden_layers: basic_config.num_hidden_layers,
            num_attention_heads: basic_config.num_attention_heads,
            num_key_value_heads: basic_config
                .num_key_value_heads
                .unwrap_or(basic_config.num_attention_heads),
            rms_norm_eps: basic_config.rms_norm_eps,
            rope_theta: basic_config.rope_theta,
            use_flash_attn,
        })
    }
}

pub struct LlamaLoader;

impl NormalModelLoader for LlamaLoader {
    fn load(
        &self,
        config: &str,
        use_flash_attn: bool,
        vb: VarBuilder,
    ) -> Result<Box<dyn NormalModel + Send + Sync>> {
        Ok(Box::new(models::llama::Llama::new(
            &LlamaBasicConfig::deserialize(config, use_flash_attn)?,
            vb,
            self.is_gptx(),
        )?))
    }
    fn load_xlora(
        &self,
        config: &str,
        use_flash_attn: bool,
        vb: VarBuilder,
        lora_config: &[(String, LoraConfig)],
        xlora_config: Option<XLoraConfig>,
        xlora_ordering: Ordering,
    ) -> Result<Box<dyn NormalModel + Send + Sync>> {
        Ok(Box::new(xlora_models::XLoraLlama::new(
            &LlamaBasicConfig::deserialize(config, use_flash_attn)?,
            vb,
            lora_config,
            xlora_config,
            xlora_ordering,
            self.is_gptx(),
        )?))
    }
    fn is_gptx(&self) -> bool {
        true
    }
}

// ======================== Mixtral loader

#[derive(Deserialize)]
pub struct MixtralBasicConfig {
    vocab_size: usize,
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: usize,
    hidden_act: Activation,
    max_position_embeddings: usize,
    rms_norm_eps: f64,
    rope_theta: f64,
    sliding_window: usize,
    num_experts_per_tok: usize,
    num_local_experts: usize,
}

impl MixtralBasicConfig {
    fn deserialize(slice: &str, use_flash_attn: bool) -> Result<models::mixtral::Config> {
        let basic_config: Self = serde_json::from_str(slice)?;
        Ok(models::mixtral::Config {
            vocab_size: basic_config.vocab_size,
            hidden_size: basic_config.hidden_size,
            intermediate_size: basic_config.intermediate_size,
            num_hidden_layers: basic_config.num_hidden_layers,
            num_attention_heads: basic_config.num_attention_heads,
            num_key_value_heads: basic_config.num_key_value_heads,
            hidden_act: basic_config.hidden_act,
            max_position_embeddings: basic_config.max_position_embeddings,
            rms_norm_eps: basic_config.rms_norm_eps,
            rope_theta: basic_config.rope_theta,
            sliding_window: basic_config.sliding_window,
            use_flash_attn,
            num_experts_per_tok: basic_config.num_experts_per_tok,
            num_local_experts: basic_config.num_local_experts,
        })
    }
}

pub struct MixtralLoader;

impl NormalModelLoader for MixtralLoader {
    fn load(
        &self,
        config: &str,
        use_flash_attn: bool,
        vb: VarBuilder,
    ) -> Result<Box<dyn NormalModel + Send + Sync>> {
        Ok(Box::new(models::mixtral::Model::new(
            &MixtralBasicConfig::deserialize(config, use_flash_attn)?,
            vb,
            self.is_gptx(),
        )?))
    }
    fn load_xlora(
        &self,
        config: &str,
        use_flash_attn: bool,
        vb: VarBuilder,
        lora_config: &[(String, LoraConfig)],
        xlora_config: Option<XLoraConfig>,
        xlora_ordering: Ordering,
    ) -> Result<Box<dyn NormalModel + Send + Sync>> {
        Ok(Box::new(xlora_models::XLoraMixtral::new(
            &MixtralBasicConfig::deserialize(config, use_flash_attn)?,
            vb,
            lora_config,
            xlora_config,
            xlora_ordering,
            self.is_gptx(),
        )?))
    }
    fn is_gptx(&self) -> bool {
        true
    }
}

// ======================== Phi2 loader

#[derive(Deserialize)]
pub struct Phi2BasicConfig {
    vocab_size: usize,
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: Option<usize>,
    hidden_act: Activation,
    max_position_embeddings: usize,
    layer_norm_eps: f64,
    tie_word_embeddings: bool,
    rope_theta: f32,
    partial_rotary_factor: f64,
    qk_layernorm: bool,
}

impl Phi2BasicConfig {
    fn deserialize(slice: &str, use_flash_attn: bool) -> Result<models::phi2::Config> {
        let basic_config: Self = serde_json::from_str(slice)?;
        Ok(models::phi2::Config {
            vocab_size: basic_config.vocab_size,
            hidden_size: basic_config.hidden_size,
            intermediate_size: basic_config.intermediate_size,
            num_hidden_layers: basic_config.num_hidden_layers,
            num_attention_heads: basic_config.num_attention_heads,
            num_key_value_heads: basic_config.num_key_value_heads,
            hidden_act: basic_config.hidden_act,
            max_position_embeddings: basic_config.max_position_embeddings,
            rope_theta: basic_config.rope_theta,
            layer_norm_eps: basic_config.layer_norm_eps,
            tie_word_embeddings: basic_config.tie_word_embeddings,
            partial_rotary_factor: basic_config.partial_rotary_factor,
            qk_layernorm: basic_config.qk_layernorm,
            use_flash_attn,
        })
    }
}

pub struct Phi2Loader;

impl NormalModelLoader for Phi2Loader {
    fn load(
        &self,
        config: &str,
        use_flash_attn: bool,
        vb: VarBuilder,
    ) -> Result<Box<dyn NormalModel + Send + Sync>> {
        Ok(Box::new(models::phi2::Model::new(
            &Phi2BasicConfig::deserialize(config, use_flash_attn)?,
            vb,
            self.is_gptx(),
        )?))
    }
    fn load_xlora(
        &self,
        config: &str,
        use_flash_attn: bool,
        vb: VarBuilder,
        lora_config: &[(String, LoraConfig)],
        xlora_config: Option<XLoraConfig>,
        xlora_ordering: Ordering,
    ) -> Result<Box<dyn NormalModel + Send + Sync>> {
        Ok(Box::new(xlora_models::XLoraPhi2::new(
            &Phi2BasicConfig::deserialize(config, use_flash_attn)?,
            vb,
            lora_config,
            xlora_config,
            xlora_ordering,
            self.is_gptx(),
        )?))
    }
    fn is_gptx(&self) -> bool {
        true
    }
}
