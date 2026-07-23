#[cfg(test)]
mod tests {
    use crate::state::AppState;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn test_per_cluster_privacy_evaluation() {
        let state = Arc::new(AppState::new_minimal_mock().await);

        // 1. Initial State: Global = false, no cluster policies
        assert!(!state.governance.is_privacy_mode_enabled(None));
        assert!(!state.governance.is_privacy_mode_enabled(Some("cluster-alpha")));
        assert!(!state.governance.is_privacy_mode_enabled(Some("cluster-beta")));

        // 2. Set per-cluster privacy policy for cluster-alpha ONLY
        state.governance.cluster_privacy_policies.insert("cluster-alpha".to_string(), true);

        // Verify cluster-alpha is air-gapped while cluster-beta remains hybrid
        assert!(state.governance.is_privacy_mode_enabled(Some("cluster-alpha")));
        assert!(!state.governance.is_privacy_mode_enabled(Some("cluster-beta")));
        assert!(!state.governance.is_privacy_mode_enabled(None));

        // 3. Enable Master Global Privacy Mode
        state.governance.privacy_mode.store(true, Ordering::Relaxed);

        // Global master override forces ALL clusters to air-gapped mode
        assert!(state.governance.is_privacy_mode_enabled(Some("cluster-alpha")));
        assert!(state.governance.is_privacy_mode_enabled(Some("cluster-beta")));
        assert!(state.governance.is_privacy_mode_enabled(None));
    }
}
