# Collection

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | identifier of the collection used, for example, in URIs | 
**title** | Option<**String**> | human readable title of the collection | [optional]
**description** | Option<**String**> | a description of the features in the collection | [optional]
**links** | [**Vec<crate::models::Link>**](link.md) |  | 
**extent** | Option<[**crate::models::Extent**](extent.md)> |  | [optional]
**item_type** | Option<**String**> | indicator about the type of the items in the collection (the default value is 'feature'). | [optional][default to feature]
**crs** | Option<**Vec<String>**> | the list of coordinate reference systems supported by the service | [optional][default to ["http://www.opengis.net/def/crs/OGC/1.3/CRS84"]]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


