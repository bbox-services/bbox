# data_api

All URIs are relative to *https://data.example.org*

Method | HTTP request | Description
------------- | ------------- | -------------
**getFeature**](data_api.md#getFeature) | **GET** /collections/{collectionId}/items/{featureId} | fetch a single feature
**getFeatures**](data_api.md#getFeatures) | **GET** /collections/{collectionId}/items | fetch features


# **getFeature**
> models::FeatureGeoJson getFeature(collection_id, feature_id)
fetch a single feature

Fetch the feature with id `featureId` in the feature collection with id `collectionId`.  Use content negotiation to request HTML or GeoJSON.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **collection_id** | **String**| local identifier of a collection | 
  **feature_id** | **String**| local identifier of a feature | 

### Return type

[**models::FeatureGeoJson**](featureGeoJSON.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/geo+json, application/json, text/html, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **getFeatures**
> models::FeatureCollectionGeoJson getFeatures(collection_id, optional)
fetch features

Fetch features of the feature collection with id `collectionId`.  Every feature in a dataset belongs to a collection. A dataset may consist of multiple feature collections. A feature collection is often a collection of features of a similar type, based on a common schema.  Use content negotiation to request HTML or GeoJSON.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **collection_id** | **String**| local identifier of a collection | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **collection_id** | **String**| local identifier of a collection | 
 **limit** | **i32**| The optional limit parameter limits the number of items that are presented in the response document.  Only items are counted that are on the first level of the collection in the response document. Nested objects contained within the explicitly requested items shall not be counted.  Minimum = 1. Maximum = 10000. Default = 10. | [default to 10]
 **bbox** | [**f64**](f64.md)| Only features that have a geometry that intersects the bounding box are selected. The bounding box is provided as four or six numbers, depending on whether the coordinate reference system includes a vertical axis (height or depth):  * Lower left corner, coordinate axis 1 * Lower left corner, coordinate axis 2 * Minimum value, coordinate axis 3 (optional) * Upper right corner, coordinate axis 1 * Upper right corner, coordinate axis 2 * Maximum value, coordinate axis 3 (optional)  The coordinate reference system of the values is WGS 84 longitude/latitude (http://www.opengis.net/def/crs/OGC/1.3/CRS84) unless a different coordinate reference system is specified in the parameter `bbox-crs`.  For WGS 84 longitude/latitude the values are in most cases the sequence of minimum longitude, minimum latitude, maximum longitude and maximum latitude. However, in cases where the box spans the antimeridian the first value (west-most box edge) is larger than the third value (east-most box edge).  If the vertical axis is included, the third and the sixth number are the bottom and the top of the 3-dimensional bounding box.  If a feature has multiple spatial geometry properties, it is the decision of the server whether only a single spatial geometry property is used to determine the extent or all relevant geometries. | 
 **datetime** | **String**| Either a date-time or an interval, open or closed. Date and time expressions adhere to RFC 3339. Open intervals are expressed using double-dots.  Examples:  * A date-time: \"2018-02-12T23:20:50Z\" * A closed interval: \"2018-02-12T00:00:00Z/2018-03-18T12:31:12Z\" * Open intervals: \"2018-02-12T00:00:00Z/..\" or \"../2018-03-18T12:31:12Z\"  Only features that have a temporal property that intersects the value of `datetime` are selected.  If a feature has multiple temporal properties, it is the decision of the server whether only a single temporal property is used to determine the extent or all relevant temporal properties. | 

### Return type

[**models::FeatureCollectionGeoJson**](featureCollectionGeoJSON.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/geo+jsonapplication/jsontext/html

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

