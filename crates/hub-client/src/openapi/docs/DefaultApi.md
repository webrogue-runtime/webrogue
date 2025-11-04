# \DefaultApi

All URIs are relative to *https://api.hub.example.com/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**auth_login**](DefaultApi.md#auth_login) | **POST** /api/v1/auth/login | User login
[**auth_signup**](DefaultApi.md#auth_signup) | **POST** /api/v1/auth/signup | User signup (registration)
[**auth_verify_email**](DefaultApi.md#auth_verify_email) | **POST** /api/v1/verify_email | Verify Email with OTP
[**get_current_user**](DefaultApi.md#get_current_user) | **GET** /api/v1/users/me | Get current user data
[**list_devices**](DefaultApi.md#list_devices) | **GET** /api/v1/devices | List user devices



## auth_login

> models::AuthLoginResponse auth_login(auth_login_request)
User login

Authenticate a user with username and password

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_login_request** | [**AuthLoginRequest**](AuthLoginRequest.md) |  | [required] |

### Return type

[**models::AuthLoginResponse**](AuthLoginResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_signup

> models::AuthSignupResponse auth_signup(auth_signup_request)
User signup (registration)

Register a new user account. May require email verifyation

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_signup_request** | [**AuthSignupRequest**](AuthSignupRequest.md) |  | [required] |

### Return type

[**models::AuthSignupResponse**](AuthSignupResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_verify_email

> models::AuthVerifyEmailResponse auth_verify_email(auth_verify_email_request)
Verify Email with OTP

Verify user registration using OTP code sent to email

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_verify_email_request** | [**AuthVerifyEmailRequest**](AuthVerifyEmailRequest.md) |  | [required] |

### Return type

[**models::AuthVerifyEmailResponse**](AuthVerifyEmailResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_current_user

> models::UserResponse get_current_user()
Get current user data

Retrieve the authenticated user's profile information

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::UserResponse**](UserResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
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

