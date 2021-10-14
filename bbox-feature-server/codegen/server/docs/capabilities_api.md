# capabilities_api

All URIs are relative to *https://data.example.org*

Method | HTTP request | Description
------------- | ------------- | -------------
**describeCollection**](capabilities_api.md#describeCollection) | **GET** /collections/{collectionId} | describe the feature collection with id `collectionId`
**getCollections**](capabilities_api.md#getCollections) | **GET** /collections | the feature collections in the dataset
**getConformanceDeclaration**](capabilities_api.md#getConformanceDeclaration) | **GET** /conformance | information about specifications that this API conforms to
**getLandingPage**](capabilities_api.md#getLandingPage) | **GET** / | landing page


# **describeCollection**
> models::Collection describeCollection(collection_id)
describe the feature collection with id `collectionId`

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **collection_id** | **String**| local identifier of a collection | 

### Return type

[**models::Collection**](collection.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, text/html, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **getCollections**
> models::Collections getCollections()
the feature collections in the dataset

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**models::Collections**](collections.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, text/html, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **getConformanceDeclaration**
> models::ConfClasses getConformanceDeclaration()
information about specifications that this API conforms to

A list of all conformance classes specified in a standard that the server conforms to.

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**models::ConfClasses**](confClasses.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, text/html, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **getLandingPage**
> models::LandingPage getLandingPage()
landing page

The landing page provides links to the API definition, the conformance statements and to the feature collections in this dataset.

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**models::LandingPage**](landingPage.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/jsontext/html

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

