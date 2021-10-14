# FeatureCollectionGeoJson

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**_type** | **String** |  | 
**features** | [**Vec<crate::models::FeatureGeoJson>**](featureGeoJSON.md) |  | 
**links** | Option<[**Vec<crate::models::Link>**](link.md)> |  | [optional]
**time_stamp** | Option<**String**> | This property indicates the time and date when the response was generated. | [optional]
**number_matched** | Option<**i32**> | The number of features of the feature type that match the selection parameters like `bbox`. | [optional]
**number_returned** | Option<**i32**> | The number of features in the feature collection.  A server may omit this information in a response, if the information about the number of features is not known or difficult to compute.  If the value is provided, the value shall be identical to the number of items in the \"features\" array. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


