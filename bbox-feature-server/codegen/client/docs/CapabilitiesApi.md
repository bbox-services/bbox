# \CapabilitiesApi

All URIs are relative to *https://data.example.org*

Method | HTTP request | Description
------------- | ------------- | -------------
[**describe_collection**](CapabilitiesApi.md#describe_collection) | **Get** /collections/{collectionId} | describe the feature collection with id `collectionId`
[**get_collections**](CapabilitiesApi.md#get_collections) | **Get** /collections | the feature collections in the dataset
[**get_conformance_declaration**](CapabilitiesApi.md#get_conformance_declaration) | **Get** /conformance | information about specifications that this API conforms to
[**get_landing_page**](CapabilitiesApi.md#get_landing_page) | **Get** / | landing page



## describe_collection

> crate::models::Collection describe_collection(collection_id)
describe the feature collection with id `collectionId`

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**collection_id** | **String** | local identifier of a collection | [required] |

### Return type

[**crate::models::Collection**](collection.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, text/html

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_collections

> crate::models::Collections get_collections()
the feature collections in the dataset

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::Collections**](collections.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, text/html

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_conformance_declaration

> crate::models::ConfClasses get_conformance_declaration()
information about specifications that this API conforms to

A list of all conformance classes specified in a standard that the server conforms to.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ConfClasses**](confClasses.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, text/html

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_landing_page

> crate::models::LandingPage get_landing_page()
landing page

The landing page provides links to the API definition, the conformance statements and to the feature collections in this dataset.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::LandingPage**](landingPage.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, text/html

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

