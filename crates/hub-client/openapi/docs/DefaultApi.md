# \DefaultApi

All URIs are relative to *https://api.hub.example.com/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**abort_wrapp_upload**](DefaultApi.md#abort_wrapp_upload) | **POST** /api/v1/wrapps/{id}/upload/abort | Abort a multipart upload
[**complete_wrapp_upload**](DefaultApi.md#complete_wrapp_upload) | **POST** /api/v1/wrapps/{id}/upload/complete | Complete a multipart upload
[**create_wrapp**](DefaultApi.md#create_wrapp) | **POST** /api/v1/wrapps | Create a wrapp
[**delete_wrapp**](DefaultApi.md#delete_wrapp) | **DELETE** /api/v1/wrapps/{id} | Delete a wrapp
[**get_wrapp**](DefaultApi.md#get_wrapp) | **GET** /api/v1/wrapps/{id} | Get a wrapp
[**get_wrapp_download_url**](DefaultApi.md#get_wrapp_download_url) | **GET** /api/v1/wrapps/{id}/download_url | Get a download URL
[**get_wrapp_part_urls**](DefaultApi.md#get_wrapp_part_urls) | **POST** /api/v1/wrapps/{id}/upload/part_urls | Get presigned part URLs
[**list_devices**](DefaultApi.md#list_devices) | **GET** /api/v1/devices | List user devices
[**list_example_wrapps**](DefaultApi.md#list_example_wrapps) | **GET** /api/v1/wrapps/examples | List example wrapps
[**list_wrapps**](DefaultApi.md#list_wrapps) | **GET** /api/v1/wrapps | List wrapps
[**publish_wrapp**](DefaultApi.md#publish_wrapp) | **POST** /api/v1/wrapps/{id}/publish | Publish a wrapp
[**replace_wrapp**](DefaultApi.md#replace_wrapp) | **POST** /api/v1/wrapps/{id}/upload | Replace a wrapp
[**unpublish_wrapp**](DefaultApi.md#unpublish_wrapp) | **DELETE** /api/v1/wrapps/{id}/publish | Unpublish a wrapp



## abort_wrapp_upload

> abort_wrapp_upload(id)
Abort a multipart upload

Abort a pending multipart upload.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **uuid::Uuid** |  | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## complete_wrapp_upload

> models::GetWrappResponse complete_wrapp_upload(id, complete_upload_request)
Complete a multipart upload

Complete a multipart upload and verify the object size.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **uuid::Uuid** |  | [required] |
**complete_upload_request** | [**CompleteUploadRequest**](CompleteUploadRequest.md) |  | [required] |

### Return type

[**models::GetWrappResponse**](GetWrappResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_wrapp

> models::CreateWrappResponse create_wrapp(create_wrapp_request)
Create a wrapp

Create wrapp metadata and start a multipart upload.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_wrapp_request** | [**CreateWrappRequest**](CreateWrappRequest.md) |  | [required] |

### Return type

[**models::CreateWrappResponse**](CreateWrappResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_wrapp

> delete_wrapp(id)
Delete a wrapp

Delete a wrapp and its stored object.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **uuid::Uuid** |  | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_wrapp

> models::GetWrappResponse get_wrapp(id)
Get a wrapp

Get a wrapp owned by the current user.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **uuid::Uuid** |  | [required] |

### Return type

[**models::GetWrappResponse**](GetWrappResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_wrapp_download_url

> models::DownloadUrlResponse get_wrapp_download_url(id)
Get a download URL

Get a public or presigned download URL for a wrapp.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **uuid::Uuid** |  | [required] |

### Return type

[**models::DownloadUrlResponse**](DownloadUrlResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_wrapp_part_urls

> models::PartUrlsResponse get_wrapp_part_urls(id, part_urls_request)
Get presigned part URLs

Get presigned UploadPart URLs for a pending multipart upload.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **uuid::Uuid** |  | [required] |
**part_urls_request** | [**PartUrlsRequest**](PartUrlsRequest.md) |  | [required] |

### Return type

[**models::PartUrlsResponse**](PartUrlsResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


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


## list_example_wrapps

> models::ListWrappsResponse list_example_wrapps()
List example wrapps

List curated public example wrapps.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListWrappsResponse**](ListWrappsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_wrapps

> models::ListWrappsResponse list_wrapps()
List wrapps

List wrapps owned by the current user.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListWrappsResponse**](ListWrappsResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## publish_wrapp

> models::PublishWrappResponse publish_wrapp(id)
Publish a wrapp

Copy a wrapp to the public bucket and return its public URL.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **uuid::Uuid** |  | [required] |

### Return type

[**models::PublishWrappResponse**](PublishWrappResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## replace_wrapp

> models::CreateWrappResponse replace_wrapp(id, replace_wrapp_request)
Replace a wrapp

Start a multipart upload that replaces the contents of an existing wrapp. The wrapp id, name, and public URL are preserved; if the wrapp is published, the public copy is updated when the upload completes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **uuid::Uuid** |  | [required] |
**replace_wrapp_request** | [**ReplaceWrappRequest**](ReplaceWrappRequest.md) |  | [required] |

### Return type

[**models::CreateWrappResponse**](CreateWrappResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unpublish_wrapp

> unpublish_wrapp(id)
Unpublish a wrapp

Remove a wrapp from the public bucket.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **uuid::Uuid** |  | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

