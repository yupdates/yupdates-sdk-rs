#[cfg(test)]
mod tests {
    use crate::test_clients;
    use yupdates::errors::Result;

    /// Add items to a pre-existing feed.
    ///
    /// Requires the YUPDATES_TEST_FEED_SPECIFIC_TOKEN and YUPDATES_TEST_RO_TOKEN environment
    /// variables.
    #[tokio::test]
    async fn basic_add_items() -> Result<()> {
        let (ro_client, _feed_client) = test_clients()?;
        ro_client.ping().await?;

        Ok(())
    }
}
