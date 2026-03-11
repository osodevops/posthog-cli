use posthog::cache::DiskCache;
use tempfile::TempDir;

#[test]
fn test_cache_set_and_get() {
    let dir = TempDir::new().unwrap();
    let cache = DiskCache::new(dir.path().to_path_buf(), 300);

    cache.set("endpoint", "params", r#"{"data": "test"}"#);
    let result = cache.get("endpoint", "params");

    assert!(result.is_some());
    assert_eq!(result.unwrap(), r#"{"data": "test"}"#);
}

#[test]
fn test_cache_miss() {
    let dir = TempDir::new().unwrap();
    let cache = DiskCache::new(dir.path().to_path_buf(), 300);

    let result = cache.get("nonexistent", "params");
    assert!(result.is_none());
}

#[test]
fn test_cache_different_keys() {
    let dir = TempDir::new().unwrap();
    let cache = DiskCache::new(dir.path().to_path_buf(), 300);

    cache.set("endpoint1", "params1", "data1");
    cache.set("endpoint2", "params2", "data2");

    assert_eq!(cache.get("endpoint1", "params1").unwrap(), "data1");
    assert_eq!(cache.get("endpoint2", "params2").unwrap(), "data2");
}

#[test]
fn test_cache_overwrite() {
    let dir = TempDir::new().unwrap();
    let cache = DiskCache::new(dir.path().to_path_buf(), 300);

    cache.set("endpoint", "params", "old_data");
    cache.set("endpoint", "params", "new_data");

    assert_eq!(cache.get("endpoint", "params").unwrap(), "new_data");
}

#[test]
fn test_cache_clear() {
    let dir = TempDir::new().unwrap();
    let cache = DiskCache::new(dir.path().to_path_buf(), 300);

    cache.set("e1", "p1", "d1");
    cache.set("e2", "p2", "d2");
    cache.set("e3", "p3", "d3");

    let cleared = cache.clear().unwrap();
    assert_eq!(cleared, 3);

    assert!(cache.get("e1", "p1").is_none());
    assert!(cache.get("e2", "p2").is_none());
}

#[test]
fn test_cache_stats() {
    let dir = TempDir::new().unwrap();
    let cache = DiskCache::new(dir.path().to_path_buf(), 300);

    let (count, _) = cache.stats();
    assert_eq!(count, 0);

    cache.set("e1", "p1", "data1");
    cache.set("e2", "p2", "data2");

    let (count, size) = cache.stats();
    assert_eq!(count, 2);
    assert!(size > 0);
}

#[test]
fn test_cache_expired_entry() {
    let dir = TempDir::new().unwrap();
    // TTL of 0 seconds = everything expires immediately
    let cache = DiskCache::new(dir.path().to_path_buf(), 0);

    cache.set("endpoint", "params", "data");

    // Sleep briefly to ensure the entry is expired
    std::thread::sleep(std::time::Duration::from_millis(10));

    let result = cache.get("endpoint", "params");
    assert!(result.is_none());
}
