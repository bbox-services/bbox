# ExtentTemporal

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**interval** | [**Vec<Vec<chrono::DateTime<chrono::Utc>>>**](array.md) | One or more time intervals that describe the temporal extent of the dataset. The value `null` is supported and indicates an open time intervall. In the Core only a single time interval is supported. Extensions may support multiple intervals. If multiple intervals are provided, the union of the intervals describes the temporal extent. | [optional] [default to None]
**trs** | **String** | Coordinate reference system of the coordinates in the temporal extent (property `interval`). The default reference system is the Gregorian calendar. In the Core this is the only supported temporal reference system. Extensions may support additional temporal reference systems and add additional enum values. | [optional] [default to Some("http://www.opengis.net/def/uom/ISO-8601/0/Gregorian".to_string())]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


