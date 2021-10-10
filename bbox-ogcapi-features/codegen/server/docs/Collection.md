# Collection

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | identifier of the collection used, for example, in URIs | 
**title** | **String** | human readable title of the collection | [optional] [default to None]
**description** | **String** | a description of the features in the collection | [optional] [default to None]
**links** | [**Vec<models::Link>**](link.md) |  | 
**extent** | [***models::Extent**](extent.md) |  | [optional] [default to None]
**item_type** | **String** | indicator about the type of the items in the collection (the default value is 'feature'). | [optional] [default to Some("feature".to_string())]
**crs** | **Vec<String>** | the list of coordinate reference systems supported by the service | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


