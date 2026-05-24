# \DefaultApi

All URIs are relative to *https://api.hub.example.com/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_devices**](DefaultApi.md#list_devices) | **GET** /api/v1/devices | List user devices



## list_devices

> models::ListDevicesResponse list_devices()
List user devices

Get all devices registered for the current user. Automatically cleans up devices registered more than 60 seconds ago.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListDevicesResponse**](ListDevicesResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

