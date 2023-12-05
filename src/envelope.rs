use std::time::Duration;

use serde::{Serialize,Deserialize};

/*
EnvelopeTelemetry { 
data: DataTelemetry 
    { 
        baseType: "MetricData", 
        baseData: Object {"metrics": Array [Object {"count": Number(1), "kind": Number(1), "max": Number(10810380288), "min": Number(10810380288), "name": String("\\Memory\\Available Bytes"), "stdDev": Number(0), "value": Number(10810380288)}], "properties": Object {}, "ver": Number(2)} 
    } 
}
*/

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct DataTelemetry {
    baseType: String,
    baseData: BaseData,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct EnvelopeTelemetry {
    pub data: BaseData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "baseType", content = "baseData")]
pub enum BaseData {
    RequestData(RequestData),
    MetricData(MetricData),
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RequestData {
    pub duration: String,
    pub name: String,
    pub url: String,
    pub id: String,
    pub responseCode: String,
    pub success: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricData {
    pub metrics: Vec<Metric>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metric {
    pub min: f32,
    pub max: f32,
    pub count: i32,
    pub name: String,
}


pub fn from_duration_string(duration_string: String) -> Duration {
    let parts: Vec<&str> = duration_string.split(&[':', '.']).collect();
    if parts.len() != 4 {
        return Duration::from_millis(0);
    }
}



#[test]
fn test_invalid_duration_string_returns_0_milliseconds() {
    let duration_string = String::from("01:02:03.003.003");
    let expected_duration = Duration::from_millis(0);

    assert_eq!(expected_duration, from_duration_string(duration_string));

}
#[test]
fn test_from_valid_duration_string() {
    let duration_string = String::from("00:00:00.003");
    let expected_duration = Duration::from_millis(3);

    assert_eq!(expected_duration, from_duration_string(duration_string));

}
