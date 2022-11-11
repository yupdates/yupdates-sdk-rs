//! These tests require the YUPDATES_TEST_FEED_SPECIFIC_TOKEN and YUPDATES_TEST_RO_TOKEN
//! environment variables.
use crate::{random_test_items, test_clients};
use yupdates::api::ReadOptions;
use yupdates::errors::{Kind, Result};

/// Add items to a pre-existing feed and read items back out with various query parameters
#[tokio::test]
async fn read_item_parameters() -> Result<()> {
    let (ro_client, feed_client) = test_clients()?;
    let (input_items, suffixes) = random_test_items(33);
    let feed_id = feed_client.new_items_all(&input_items, 128).await?;

    // The basic, most-recent items query:
    let most_recent_ten_items = ro_client.read_items(&feed_id).await?;
    assert_eq!(most_recent_ten_items.len(), 10);
    for (i, feed_item) in most_recent_ten_items.iter().enumerate() {
        let expected_suffix = suffixes.get(i).unwrap();
        assert_eq!(feed_item.title, format!("title-{}", expected_suffix));
        assert_eq!(feed_item.content, None);
    }

    // With content:
    let options = ReadOptions {
        include_item_content: true,
        ..Default::default()
    };
    let ten_items_with_content = ro_client
        .read_items_with_options(&feed_id, &options)
        .await?;
    assert_eq!(ten_items_with_content.len(), 10);
    for (i, feed_item) in ten_items_with_content.iter().enumerate() {
        let expected_suffix = suffixes.get(i).unwrap();
        assert_eq!(feed_item.title, format!("title-{}", expected_suffix));
        assert_eq!(
            feed_item.content,
            Some(format!("content-{}", expected_suffix))
        );
    }

    // Read all of the ones we just added. We can test more queries/bounds in the future with
    // unique feeds per test (new-feed API).
    let options = ReadOptions {
        max_items: 33,
        ..Default::default()
    };
    let all_items = ro_client
        .read_items_with_options(&feed_id, &options)
        .await?;
    assert_eq!(all_items.len(), 33);
    let mut tenth_item_time = None;
    for (i, feed_item) in all_items.iter().enumerate() {
        let expected_suffix = suffixes.get(i).unwrap();
        assert_eq!(feed_item.title, format!("title-{}", expected_suffix));
        assert_eq!(feed_item.content, None);
        assert_eq!(
            feed_item.canonical_url,
            format!("https://www.example.com/{}", expected_suffix)
        );
        if i == 9 {
            tenth_item_time = Some(feed_item.item_time.clone());
        }
    }

    // Query for items 11-23:
    let options = ReadOptions {
        max_items: 13,
        item_time_before: Some(tenth_item_time.clone().unwrap()),
        ..Default::default()
    };
    let middle_items = ro_client
        .read_items_with_options(&feed_id, &options)
        .await?;
    assert_eq!(middle_items.len(), 13);
    for (i, feed_item) in middle_items.iter().enumerate() {
        let expected_suffix = suffixes.get(i + 10).unwrap();
        assert_eq!(feed_item.title, format!("title-{}", expected_suffix));
    }

    // Query for items 5-9:
    let options = ReadOptions {
        max_items: 5,
        item_time_after: Some(tenth_item_time.unwrap()),
        ..Default::default()
    };
    let nine_to_five = ro_client
        .read_items_with_options(&feed_id, &options)
        .await?;
    assert_eq!(nine_to_five.len(), 5);
    for (i, feed_item) in nine_to_five.iter().enumerate() {
        let expected_suffix = suffixes.get(i + 4).unwrap();
        assert_eq!(feed_item.title, format!("title-{}", expected_suffix));
    }

    Ok(())
}

/// Exercise invalid options that are caught client-side
#[tokio::test]
async fn illegal_read_items() -> Result<()> {
    let feed_id = "02fb24a4478462a4491067224b66d9a8b2338ddca2737".to_string();
    let (ro_client, _) = test_clients()?;

    // Simultaneous use of `item_time_after` and `item_time_before`:
    let options = ReadOptions {
        max_items: 5,
        item_time_after: Some("1234567890".to_string()),
        item_time_before: Some("1234567890".to_string()),
        ..Default::default()
    };
    let result = ro_client.read_items_with_options(&feed_id, &options).await;
    assert!(result.is_err());
    match result.unwrap_err().kind {
        Kind::IllegalParameter(text) => {
            assert!(text.contains("cannot simultaneously query"))
        }
        e => {
            panic!("unexpected error type: {:?}", e)
        }
    }

    // Querying for more than 50:
    let options = ReadOptions {
        max_items: 51,
        ..Default::default()
    };
    let result = ro_client.read_items_with_options(&feed_id, &options).await;
    assert!(result.is_err());
    match result.unwrap_err().kind {
        Kind::IllegalParameter(text) => {
            assert!(text.contains("1 to 50"))
        }
        e => {
            panic!("unexpected error type: {:?}", e)
        }
    }

    // Querying for more than 10 when `include_item_content` is true:
    let options = ReadOptions {
        max_items: 11,
        include_item_content: true,
        ..Default::default()
    };
    let result = ro_client.read_items_with_options(&feed_id, &options).await;
    assert!(result.is_err());
    match result.unwrap_err().kind {
        Kind::IllegalParameter(text) => {
            assert!(text.contains("1 to 10 when"))
        }
        e => {
            panic!("unexpected error type: {:?}", e)
        }
    }

    // Illegal item_time (`item_time_after` is not a number):
    let options = ReadOptions {
        max_items: 5,
        item_time_after: Some("1234567890x".to_string()),
        ..Default::default()
    };
    let result = ro_client.read_items_with_options(&feed_id, &options).await;
    assert!(result.is_err());
    match result.unwrap_err().kind {
        Kind::IllegalParameter(text) => {
            assert!(text.contains("invalid u64"))
        }
        e => {
            panic!("unexpected error type: {:?}", e)
        }
    }

    // Illegal item_time (`item_time_before` is not a number):
    let options = ReadOptions {
        max_items: 5,
        item_time_before: Some("1234567890x".to_string()),
        ..Default::default()
    };
    let result = ro_client.read_items_with_options(&feed_id, &options).await;
    assert!(result.is_err());
    match result.unwrap_err().kind {
        Kind::IllegalParameter(text) => {
            assert!(text.contains("invalid u64"))
        }
        e => {
            panic!("unexpected error type: {:?}", e)
        }
    }

    // Illegal item_time (base ms is too large):
    let options = ReadOptions {
        max_items: 5,
        item_time_before: Some("99999999999990".to_string()),
        ..Default::default()
    };
    let result = ro_client.read_items_with_options(&feed_id, &options).await;
    assert!(result.is_err());
    match result.unwrap_err().kind {
        Kind::IllegalParameter(text) => {
            assert!(text.contains("may not be larger than"))
        }
        e => {
            panic!("unexpected error type: {:?}", e)
        }
    }

    // Illegal item_time (suffix is too large):
    let options = ReadOptions {
        max_items: 5,
        item_time_before: Some("123456789.1234567".to_string()),
        ..Default::default()
    };
    let result = ro_client.read_items_with_options(&feed_id, &options).await;
    assert!(result.is_err());
    match result.unwrap_err().kind {
        Kind::IllegalParameter(text) => {
            assert!(text.contains("may not be larger than"))
        }
        e => {
            panic!("unexpected error type: {:?}", e)
        }
    }

    Ok(())
}
