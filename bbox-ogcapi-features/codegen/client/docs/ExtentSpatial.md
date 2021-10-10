# ExtentSpatial

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bbox** | Option<[**Vec<Vec<f32>>**](array.md)> | One or more bounding boxes that describe the spatial extent of the dataset. In the Core only a single bounding box is supported. Extensions may support additional areas. If multiple areas are provided, the union of the bounding boxes describes the spatial extent. | [optional]
**crs** | Option<**String**> | Coordinate reference system of the coordinates in the spatial extent (property `bbox`). The default reference system is WGS 84 longitude/latitude. In the Core this is the only supported coordinate reference system. Extensions may support additional coordinate reference systems and add additional enum values. | [optional][default to Crs_HttpWwwOpengisNetDefCrsOGC13CRS84]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


