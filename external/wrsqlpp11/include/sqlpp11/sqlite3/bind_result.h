#pragma once

/*
 * Copyright (c) 2013 - 2015, Roland Bock
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 *
 *   Redistributions of source code must retain the above copyright notice, this
 *   list of conditions and the following disclaimer.
 *
 *   Redistributions in binary form must reproduce the above copyright notice, this
 *   list of conditions and the following disclaimer in the documentation and/or
 *   other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR
 * ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
 * ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#include <memory>

#include <sqlpp11/exception.h>
#include <sqlpp11/sqlite3/export.h>
#include <sqlpp11/sqlite3/prepared_statement_handle.h>

#ifdef _MSC_VER
#include <iso646.h>
#pragma warning(push)
#pragma warning(disable : 4251)
#endif

namespace sqlpp
{
  namespace sqlite3
  {
    namespace detail
    {
      inline auto check_first_digit(const char* text, bool digitFlag) -> bool
      {
        if (digitFlag)
        {
          if (not std::isdigit(*text))
          {
            return false;
          }
        }
        else
        {
          if (std::isdigit(*text) or *text == '\0')
          {
            return false;
          }
        }
        return true;
      }

      inline auto check_date_digits(const char* text) -> bool
      {
        for (const auto digitFlag : {true, true, true, true, false, true, true, false, true, true})  // YYYY-MM-DD
        {
          if (not check_first_digit(text, digitFlag))
            return false;
          ++text;
        }
        return true;
      }

      inline auto check_time_digits(const char* text) -> bool
      {
        for (const auto digitFlag : {true, true, false, true, true, false, true, true})  // hh:mm:ss
        {
          if (not check_first_digit(text, digitFlag))
            return false;
          ++text;
        }
        return true;
      }

      inline auto check_ms_digits(const char* text) -> bool
      {
        for (const auto digitFlag : {true, true, true})
        {
          if (not check_first_digit(text, digitFlag))
            return false;
          ++text;
        }
        return true;
      }
    }  // namespace detail

    class SQLPP11_SQLITE3_EXPORT bind_result_t
    {
      std::shared_ptr<detail::prepared_statement_handle_t> _handle;

    public:
      bind_result_t() = default;
      bind_result_t(const std::shared_ptr<detail::prepared_statement_handle_t>& handle) : _handle(handle)
      {
        if (_handle and _handle->debug)
          core::wrerr << "Sqlite3 debug: Constructing bind result, using handle at " << _handle.get() << std::endl;
      }

      bind_result_t(const bind_result_t&) = delete;
      bind_result_t(bind_result_t&& rhs) = default;
      bind_result_t& operator=(const bind_result_t&) = delete;
      bind_result_t& operator=(bind_result_t&&) = default;
      ~bind_result_t() = default;

      bool operator==(const bind_result_t& rhs) const
      {
        return _handle == rhs._handle;
      }

      template <typename ResultRow>
      void next(ResultRow& result_row)
      {
        if (!_handle)
        {
          result_row._invalidate();
          return;
        }

        if (next_impl())
        {
          if (not result_row)
          {
            result_row._validate();
          }
          result_row._bind(*this);
        }
        else
        {
          if (result_row)
            result_row._invalidate();
        }
      }

      void _bind_boolean_result(size_t index, signed char* value, bool* is_null)
      {
        if (_handle->debug)
          core::wrerr << "Sqlite3 debug: binding boolean result " << *value << " at index: " << index << std::endl;

        *value = static_cast<signed char>(sqlite3wr_column_int(_handle->sqlite_statement, static_cast<int>(index)));
        *is_null = sqlite3wr_column_type(_handle->sqlite_statement, static_cast<int>(index)) == SQLITE_NULL;
      }

      void _bind_floating_point_result(size_t index, double* value, bool* is_null)
      {
        if (_handle->debug)
          core::wrerr << "Sqlite3 debug: binding floating_point result " << *value << " at index: " << index
                      << std::endl;

        switch (sqlite3wr_column_type(_handle->sqlite_statement, static_cast<int>(index)))
        {
          case (SQLITE3_TEXT):
            *value = atof(reinterpret_cast<const char*>(
                sqlite3wr_column_text(_handle->sqlite_statement, static_cast<int>(index))));
            break;
          default:
            *value = sqlite3wr_column_double(_handle->sqlite_statement, static_cast<int>(index));
        }
        *is_null = sqlite3wr_column_type(_handle->sqlite_statement, static_cast<int>(index)) == SQLITE_NULL;
      }

      void _bind_integral_result(size_t index, int64_t* value, bool* is_null)
      {
        if (_handle->debug)
          core::wrerr << "Sqlite3 debug: binding integral result " << *value << " at index: " << index << std::endl;

        *value = sqlite3wr_column_int64(_handle->sqlite_statement, static_cast<int>(index));
        *is_null = sqlite3wr_column_type(_handle->sqlite_statement, static_cast<int>(index)) == SQLITE_NULL;
      }

      void _bind_unsigned_integral_result(size_t index, uint64_t* value, bool* is_null)
      {
        if (_handle->debug)
          core::wrerr << "Sqlite3 debug: binding unsigned integral result " << *value << " at index: " << index
                      << std::endl;

        *value = static_cast<uint64_t>(sqlite3wr_column_int64(_handle->sqlite_statement, static_cast<int>(index)));
        *is_null = sqlite3wr_column_type(_handle->sqlite_statement, static_cast<int>(index)) == SQLITE_NULL;
      }

      void _bind_text_result(size_t index, const char** value, size_t* len)
      {
        if (_handle->debug)
          core::wrerr << "Sqlite3 debug: binding text result at index: " << index << std::endl;

        *value =
            (reinterpret_cast<const char*>(sqlite3wr_column_text(_handle->sqlite_statement, static_cast<int>(index))));
        *len = static_cast<size_t>(sqlite3wr_column_bytes(_handle->sqlite_statement, static_cast<int>(index)));
      }

      void _bind_blob_result(size_t index, const uint8_t** value, size_t* len)
      {
        if (_handle->debug)
          core::wrerr << "Sqlite3 debug: binding text result at index: " << index << std::endl;

        *value = (reinterpret_cast<const uint8_t*>(
            sqlite3wr_column_blob(_handle->sqlite_statement, static_cast<int>(index))));
        *len = static_cast<size_t>(sqlite3wr_column_bytes(_handle->sqlite_statement, static_cast<int>(index)));
      }

    private:
      bool next_impl()
      {
        if (_handle->debug)
          core::wrerr << "Sqlite3 debug: Accessing next row of handle at " << _handle.get() << std::endl;

        auto rc = sqlite3wr_step(_handle->sqlite_statement);

        switch (rc)
        {
          case SQLITE_ROW:
            return true;
          case SQLITE_DONE:
            return false;
          default:
            throw sqlpp::exception("Sqlite3 error: Unexpected return value for sqlite3wr_step()");
        }
      }
    };
  }  // namespace sqlite3
}  // namespace sqlpp

#ifdef _MSC_VER
#pragma warning(pop)
#endif
