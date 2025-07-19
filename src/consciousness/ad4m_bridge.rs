use ad4m_client::Ad4mClient;

/// Bridge to interact with the AD4M network
pub struct Ad4mBridge {
    _client: Option<Ad4mClient>,
}

impl Ad4mBridge {
    /// Create a new bridge instance
    pub fn new() -> Self {
        Self { _client: None }
    }
}
