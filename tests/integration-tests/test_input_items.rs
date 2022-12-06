//! These tests require the YUPDATES_TEST_FEED_SPECIFIC_TOKEN and YUPDATES_TEST_RO_TOKEN
//! environment variables.
use crate::{random_test_items, test_clients};
use yupdates::errors::{Kind, Result};

/// Add items to a pre-existing feed and read 10 items back out.
#[tokio::test]
async fn basic_add_items() -> Result<()> {
    let (ro_client, feed_client) = test_clients()?;
    let (input_items, suffixes) = random_test_items(24);

    // Can't send more than 10 at once:
    let result = feed_client.new_items(&input_items).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err().kind,
        Kind::IllegalParameter { .. }
    ));

    // Send more than 10 at once by breaking them up into separate calls:
    let feed_id = feed_client.new_items_all(&input_items, 128).await?;

    // Read 10 back, content is not expected:
    let most_recent_ten_items = ro_client.read_items(&feed_id).await?;
    assert_eq!(most_recent_ten_items.len(), 10);
    for (idx, feed_item) in most_recent_ten_items.iter().enumerate() {
        let expected_suffix = suffixes.get(idx).unwrap();
        assert_eq!(feed_item.title, format!("title-{}", expected_suffix));
        assert_eq!(feed_item.content, None);
        // Files are only present when content is requested:
        if let Some(files) = &feed_item.associated_files {
            assert!(files.is_empty());
        }
    }

    Ok(())
}
