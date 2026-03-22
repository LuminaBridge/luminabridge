//! Pricing module for LuminaBridge
//!
//! Handles cost calculation based on model pricing.
//! 处理基于模型定价的成本计算。

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Pricing model for a specific model
/// 特定模型的定价模型
#[derive(Debug, Clone)]
pub struct PricingModel {
    /// Model identifier
    /// 模型标识符
    pub model_id: String,
    
    /// Provider name
    /// 提供商名称
    pub provider: String,
    
    /// Price per 1K prompt tokens (in USD)
    /// 每 1K prompt tokens 价格（美元）
    pub prompt_price_per_1k: f64,
    
    /// Price per 1K completion tokens (in USD)
    /// 每 1K completion tokens 价格（美元）
    pub completion_price_per_1k: f64,
}

impl PricingModel {
    /// Calculate the total cost for a request
    /// 计算请求的总成本
    ///
    /// # Arguments
    ///
    /// * `prompt_tokens` - Number of prompt tokens used
    /// * `completion_tokens` - Number of completion tokens used
    ///
    /// # Returns
    ///
    /// Total cost in USD
    pub fn calculate_cost(&self, prompt_tokens: i64, completion_tokens: i64) -> f64 {
        let prompt_cost = (prompt_tokens as f64 / 1000.0) * self.prompt_price_per_1k;
        let completion_cost = (completion_tokens as f64 / 1000.0) * self.completion_price_per_1k;
        prompt_cost + completion_cost
    }
}

/// Built-in pricing models
/// 内置定价模型
static BUILTIN_PRICING: Lazy<HashMap<&'static str, PricingModel>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // GPT-4 models
    map.insert("gpt-4", PricingModel {
        model_id: "gpt-4".to_string(),
        provider: "openai".to_string(),
        prompt_price_per_1k: 0.03,
        completion_price_per_1k: 0.06,
    });
    
    map.insert("gpt-4-32k", PricingModel {
        model_id: "gpt-4-32k".to_string(),
        provider: "openai".to_string(),
        prompt_price_per_1k: 0.06,
        completion_price_per_1k: 0.12,
    });
    
    map.insert("gpt-4-turbo", PricingModel {
        model_id: "gpt-4-turbo".to_string(),
        provider: "openai".to_string(),
        prompt_price_per_1k: 0.01,
        completion_price_per_1k: 0.03,
    });
    
    map.insert("gpt-4o", PricingModel {
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
        prompt_price_per_1k: 0.005,
        completion_price_per_1k: 0.015,
    });
    
    map.insert("gpt-4o-mini", PricingModel {
        model_id: "gpt-4o-mini".to_string(),
        provider: "openai".to_string(),
        prompt_price_per_1k: 0.00015,
        completion_price_per_1k: 0.0006,
    });
    
    // GPT-3.5-Turbo
    map.insert("gpt-3.5-turbo", PricingModel {
        model_id: "gpt-3.5-turbo".to_string(),
        provider: "openai".to_string(),
        prompt_price_per_1k: 0.0015,
        completion_price_per_1k: 0.002,
    });
    
    // Claude-3 models
    map.insert("claude-3-opus", PricingModel {
        model_id: "claude-3-opus".to_string(),
        provider: "anthropic".to_string(),
        prompt_price_per_1k: 0.015,
        completion_price_per_1k: 0.075,
    });
    
    map.insert("claude-3-sonnet", PricingModel {
        model_id: "claude-3-sonnet".to_string(),
        provider: "anthropic".to_string(),
        prompt_price_per_1k: 0.003,
        completion_price_per_1k: 0.015,
    });
    
    map.insert("claude-3-haiku", PricingModel {
        model_id: "claude-3-haiku".to_string(),
        provider: "anthropic".to_string(),
        prompt_price_per_1k: 0.00025,
        completion_price_per_1k: 0.00125,
    });
    
    map.insert("claude-3-5-sonnet", PricingModel {
        model_id: "claude-3-5-sonnet".to_string(),
        provider: "anthropic".to_string(),
        prompt_price_per_1k: 0.003,
        completion_price_per_1k: 0.015,
    });
    
    // Gemini models
    map.insert("gemini-pro", PricingModel {
        model_id: "gemini-pro".to_string(),
        provider: "google".to_string(),
        prompt_price_per_1k: 0.00025,
        completion_price_per_1k: 0.0005,
    });
    
    map.insert("gemini-1.5-pro", PricingModel {
        model_id: "gemini-1.5-pro".to_string(),
        provider: "google".to_string(),
        prompt_price_per_1k: 0.0035,
        completion_price_per_1k: 0.0105,
    });
    
    map.insert("gemini-1.5-flash", PricingModel {
        model_id: "gemini-1.5-flash".to_string(),
        provider: "google".to_string(),
        prompt_price_per_1k: 0.000075,
        completion_price_per_1k: 0.0003,
    });
    
    map
});

/// Get built-in pricing for a model
/// 获取模型的内置定价
///
/// # Arguments
///
/// * `model` - Model name (case-insensitive matching)
///
/// # Returns
///
/// Pricing model if found, None otherwise
pub fn get_builtin_pricing(model: &str) -> Option<PricingModel> {
    let model_lower = model.to_lowercase();
    
    // Try exact match first
    if let Some(pricing) = BUILTIN_PRICING.get(model_lower.as_str()) {
        return Some(pricing.clone());
    }
    
    // Try prefix match for model variants
    for (key, pricing) in BUILTIN_PRICING.iter() {
        if model_lower.starts_with(*key) {
            return Some(pricing.clone());
        }
    }
    
    None
}

/// Calculate cost for a request
/// 计算请求的成本
///
/// # Arguments
///
/// * `model` - Model name
/// * `prompt_tokens` - Number of prompt tokens
/// * `completion_tokens` - Number of completion tokens
///
/// # Returns
///
/// Cost in USD, or 0.0 if model pricing not found
pub fn calculate_cost(model: &str, prompt_tokens: i64, completion_tokens: i64) -> f64 {
    match get_builtin_pricing(model) {
        Some(pricing) => pricing.calculate_cost(prompt_tokens, completion_tokens),
        None => 0.0, // Default to 0 if pricing not found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpt4_pricing() {
        let pricing = get_builtin_pricing("gpt-4").unwrap();
        assert_eq!(pricing.prompt_price_per_1k, 0.03);
        assert_eq!(pricing.completion_price_per_1k, 0.06);
        
        // Test cost calculation: 1000 prompt + 500 completion
        let cost = pricing.calculate_cost(1000, 500);
        assert!((cost - 0.06).abs() < 0.001); // $0.03 + $0.03 = $0.06
    }

    #[test]
    fn test_gpt35_turbo_pricing() {
        let pricing = get_builtin_pricing("gpt-3.5-turbo").unwrap();
        assert_eq!(pricing.prompt_price_per_1k, 0.0015);
        assert_eq!(pricing.completion_price_per_1k, 0.002);
    }

    #[test]
    fn test_claude3_pricing() {
        let pricing = get_builtin_pricing("claude-3-sonnet").unwrap();
        assert_eq!(pricing.prompt_price_per_1k, 0.003);
        assert_eq!(pricing.completion_price_per_1k, 0.015);
    }

    #[test]
    fn test_model_not_found() {
        let pricing = get_builtin_pricing("unknown-model");
        assert!(pricing.is_none());
    }

    #[test]
    fn test_prefix_match() {
        // Should match gpt-4 for gpt-4-0613 variant
        let pricing = get_builtin_pricing("gpt-4-0613");
        assert!(pricing.is_some());
    }

    #[test]
    fn test_calculate_cost_helper() {
        let cost = calculate_cost("gpt-4", 2000, 1000);
        // $0.03 * 2 + $0.06 * 1 = $0.12
        assert!((cost - 0.12).abs() < 0.001);
    }
}
