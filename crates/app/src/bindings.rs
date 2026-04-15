use crate::config::AppConfig;
use anyhow::{Result, bail};
use key_b0x_core::BindingId;
use key_b0x_platform::NormalizedKey;
use std::collections::HashMap;

pub struct ResolvedBindings {
    bindings: HashMap<NormalizedKey, BindingId>,
}

impl ResolvedBindings {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let mut bindings = HashMap::new();

        for binding in BindingId::ALL {
            let Some(key) = config.bindings.get(&binding) else {
                bail!("missing binding for {}", binding.label());
            };

            if let Some(existing) = bindings.insert(*key, binding) {
                bail!(
                    "duplicate key assignment: {} is assigned to {} and {}",
                    key,
                    existing.label(),
                    binding.label()
                );
            }
        }

        Ok(Self { bindings })
    }

    pub fn lookup(&self, key: NormalizedKey) -> Option<BindingId> {
        self.bindings.get(&key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use key_b0x_platform::NormalizedKey;

    #[test]
    fn resolved_bindings_reject_duplicates() {
        let mut config = AppConfig::default();
        config.bindings.insert(BindingId::A, NormalizedKey::KeyM);
        config.bindings.insert(BindingId::B, NormalizedKey::KeyM);

        assert!(ResolvedBindings::new(&config).is_err());
    }
}
