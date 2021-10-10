# FeatureCollectionGeoJson

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**_type** | **String** |  | 
**features** | [**Vec<models::FeatureGeoJson>**](featureGeoJSON.md) |  | 
**links** | [**Vec<models::Link>**](link.md) |  | [optional] [default to None]
**time_stamp** | [**chrono::DateTime<chrono::Utc>**](DateTime.md) | This property indicates the time and date when the response was generated. | [optional] [default to None]
**number_matched** | **usize** | The number of features of the feature type that match the selection parameters like `bbox`. | [optional] [default to None]
**number_returned** | **usize** | The number of features in the feature collection.  A server may omit this information in a response, if the information about the number of features is not known or difficult to compute.  If the value is provided, the value shall be identical to the number of items in the \"features\" array. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


