#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use ores_rl_lib_core::RateLimitConfigV1;
    use std::fs;
    use std::path::PathBuf;

    fn repo_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../consumers")
            .join(name)
    }

    #[test]
    fn fiducia_client_config_is_client_only_and_server_safe() {
        let config = RateLimitConfigV1::load_from_repo_root(repo_path("fiducia-clients"))
            .expect("fiducia client config must be admitted");
        let client = config.client_view().expect("client projection must exist");
        assert_eq!(client.default_policy_id.as_deref(), Some("repo-default"));
        assert_eq!(client.policies.len(), 1);
        assert!(config.server_view().is_err());
        assert!(config.edge_view().is_err());
    }

    #[test]
    fn opto_same_root_combined_config_keeps_role_projections_distinct() {
        let config = RateLimitConfigV1::load_from_repo_root(repo_path("opto-sync-monorepo"))
            .expect("combined config must be admitted");
        let client = config.client_view().expect("client projection must exist");
        let server = config.server_view().expect("server projection must exist");
        assert_eq!(client.root, ".");
        assert_eq!(server.server.root, ".");
        assert_eq!(client.policies.len(), 1);
        assert_eq!(server.policies.len(), 1);
    }

    #[test]
    fn layout_role_contradiction_fails_closed() {
        let source = fs::read_to_string(repo_path("fiducia-clients").join(".ores-rl.toml"))
            .expect("read client config");
        let mutated = source.replace("layout = \"client-only\"", "layout = \"combined\"");
        let error = RateLimitConfigV1::from_toml_str(&mutated)
            .expect_err("combined layout without server role must fail");
        assert!(error.to_string().contains("presence of [client] and [server]"));
    }

    #[test]
    fn unknown_top_level_key_fails_closed() {
        let source = fs::read_to_string(repo_path("fiducia-clients").join(".ores-rl.toml"))
            .expect("read client config");
        let mutated = source.replacen(
            "schemaVersion = \"ores.rate-limit.config.v1\"",
            "schemaVersion = \"ores.rate-limit.config.v1\"\nunexpected = true",
            1,
        );
        let error = RateLimitConfigV1::from_toml_str(&mutated)
            .expect_err("unknown key must fail closed");
        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn secret_reference_must_be_an_environment_name() {
        let source = fs::read_to_string(repo_path("opto-sync-monorepo").join(".ores-rl.toml"))
            .expect("read combined config");
        let mutated = source.replace(
            "keyHmacEnv = \"ORES_RL_HMAC_KEY\"",
            "keyHmacEnv = \"https://secret.invalid/token\"",
        );
        let error = RateLimitConfigV1::from_toml_str(&mutated)
            .expect_err("literal secret reference must fail closed");
        assert!(error.to_string().contains("environment-variable name"));
    }

    #[test]
    fn strict_consistency_cannot_silently_use_local_backend() {
        let source = fs::read_to_string(repo_path("opto-sync-monorepo").join(".ores-rl.toml"))
            .expect("read combined config");
        let mutated = source
            .replace("consistencyMode = \"advisory\"", "consistencyMode = \"strict\"")
            .replace("backendFailureMode = \"fail-open\"", "backendFailureMode = \"fail-closed\"");
        let error = RateLimitConfigV1::from_toml_str(&mutated)
            .expect_err("strict consistency with local backend must fail");
        assert!(error.to_string().contains("strict consistency requires the redis"));
    }
}
