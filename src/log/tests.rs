use tempfile::TempDir;

use super::*;

#[test]
fn unique_segment_name() {
    for _ in 0..100 {
        assert_ne!(Segment::gen_name(), Segment::gen_name());
    }
}

#[test]
fn segment_sanity_check() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut seg = Segment::new(temp_dir.path())?;
    let seg_path = seg.full_path.clone();

    seg.set("key1".to_owned(), "value1".to_owned())?;
    seg.set("key2".to_owned(), "value2".to_owned())?;
    seg.set("key3".to_owned(), "value3".to_owned())?;

    assert_eq!(seg.get("key1")?.unwrap(), "value1");
    assert_eq!(seg.get("key2")?.unwrap(), "value2");
    assert_eq!(seg.get("key3")?.unwrap(), "value3");
    assert_eq!(seg.get("key4")?, None);

    seg.remove("key2")?;
    assert_eq!(seg.get("key1")?.unwrap(), "value1");
    assert_eq!(seg.get("key2")?, None);
    assert_eq!(seg.get("key3")?.unwrap(), "value3");
    assert_eq!(seg.get("key4")?, None);

    drop(seg);

    let mut seg = Segment::open(seg_path)?;
    assert_eq!(seg.get("key1")?.unwrap(), "value1");
    assert_eq!(seg.get("key2")?, None);
    assert_eq!(seg.get("key3")?.unwrap(), "value3");
    assert_eq!(seg.get("key4")?, None);

    Ok(())
}

// Should get previously stored value.
#[test]
fn segment_get_stored_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut seg = Segment::new(temp_dir.path())?;
    let seg_path = seg.full_path.clone();

    seg.set("key1".to_owned(), "value1".to_owned())?;
    seg.set("key2".to_owned(), "value2".to_owned())?;

    assert_eq!(seg.get("key1")?, Some("value1".to_owned()));
    assert_eq!(seg.get("key2")?, Some("value2".to_owned()));

    // Open from disk again and check persistent data.
    drop(seg);
    let mut store = Segment::open(seg_path)?;
    assert_eq!(store.get("key1")?, Some("value1".to_owned()));
    assert_eq!(store.get("key2")?, Some("value2".to_owned()));

    Ok(())
}

// Should overwrite existent value.
#[test]
fn segment_overwrite_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut seg = Segment::new(temp_dir.path())?;
    let seg_path = seg.full_path.clone();

    seg.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(seg.get("key1")?, Some("value1".to_owned()));
    seg.set("key1".to_owned(), "value2".to_owned())?;
    assert_eq!(seg.get("key1")?, Some("value2".to_owned()));
    for i in 0..10_000 {
        seg.set(format!("k{}k", i), format!("V{}V", i))?;
    }
    for i in 0..5_000 {
        seg.set(format!("k{}k", i), format!("A{}A", i))?;
    }
    for i in 0..5_000 {
        assert_eq!(seg.get(&format!("k{}k", i))?, Some(format!("A{}A", i)));
    }
    for i in 5_000..10_000 {
        assert_eq!(seg.get(&format!("k{}k", i))?, Some(format!("V{}V", i)));
    }

    // Open from disk again and check persistent data.
    drop(seg);
    let mut seg = Segment::open(seg_path)?;
    assert_eq!(seg.get("key1")?, Some("value2".to_owned()));
    seg.set("key1".to_owned(), "value3".to_owned())?;
    assert_eq!(seg.get("key1")?, Some("value3".to_owned()));
    for i in 0..5_000 {
        assert_eq!(seg.get(&format!("k{}k", i))?, Some(format!("A{}A", i)));
    }
    for i in 5_000..10_000 {
        assert_eq!(seg.get(&format!("k{}k", i))?, Some(format!("V{}V", i)));
    }

    Ok(())
}

// Should get `None` when getting a non-existent key.
#[test]
fn segment_get_non_existent_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut seg = Segment::new(temp_dir.path())?;
    let seg_path = seg.full_path.clone();

    seg.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(seg.get("key2")?, None);

    // Open from disk again and check persistent data.
    drop(seg);
    let mut store = Segment::open(seg_path)?;
    assert_eq!(store.get("key2")?, None);

    Ok(())
}

#[test]
fn segment_remove_key() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut seg = Segment::new(temp_dir.path())?;
    seg.set("key1".to_owned(), "value1".to_owned())?;
    assert!(seg.remove("key1").is_ok());
    assert_eq!(seg.get("key1")?, None);
    Ok(())
}
