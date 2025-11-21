use serde_json::Value;

#[tokio::test]
async fn parse_sample_geojson_has_features() {
    let s = include_str!("../tests/fixtures/sample_geo.json");
    let v: Value = serde_json::from_str(s).expect("parse json");
    let features = v
        .get("features")
        .and_then(|f| f.as_array())
        .expect("features array");
    assert_eq!(features.len(), 1);

    let first = &features[0];
    assert_eq!(
        first.get("id").and_then(|id| id.as_str()).unwrap(),
        "usgs_test_1"
    );

    // verify geometry coordinates shape
    let coords = first
        .get("geometry")
        .and_then(|g| g.get("coordinates"))
        .and_then(|c| c.as_array())
        .expect("coords array");
    assert_eq!(coords.len(), 3);
}
