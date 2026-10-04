/* librcekunit C ABI. Auto-generated. Do not edit. */

#ifndef LIBRCEKUNIT_H
#define LIBRCEKUNIT_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum LrStatus {
    LR_STATUS_OK = 0,
    LR_STATUS_INVALID_ARG = 1,
    LR_STATUS_CONFIG = 2,
    LR_STATUS_AUTH = 3,
    LR_STATUS_NETWORK = 4,
    LR_STATUS_API = 5,
    LR_STATUS_IO = 6,
    LR_STATUS_INTERNAL = 7,
    LR_STATUS_PANIC = 8,
} LrStatus;

typedef struct LrClient LrClient;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

void lr_string_free(char *ptr);

void lr_bytes_free(uint8_t *ptr, uintptr_t len);

enum LrStatus lr_client_new(const char *base_url,
                            const char *cookie_file,
                            const char *user_agent,
                            uint64_t timeout_secs,
                            struct LrClient **out_handle,
                            char **out_error);

void lr_client_free(struct LrClient *handle);

enum LrStatus lr_client_login(struct LrClient *handle,
                              const char *email,
                              const char *password,
                              char **out_error);

enum LrStatus lr_client_logout(struct LrClient *handle, char **out_error);

enum LrStatus lr_client_index(struct LrClient *handle, char **out_body, char **out_error);

enum LrStatus lr_client_store(struct LrClient *handle,
                              const char *form_json,
                              char **out_body,
                              char **out_error);

enum LrStatus lr_client_show(struct LrClient *handle,
                             uint64_t id,
                             char **out_body,
                             char **out_error);

enum LrStatus lr_client_update(struct LrClient *handle,
                               uint64_t id,
                               const char *form_json,
                               char **out_body,
                               char **out_error);

enum LrStatus lr_client_destroy(struct LrClient *handle,
                                uint64_t id,
                                char **out_body,
                                char **out_error);

enum LrStatus lr_client_delete_all(struct LrClient *handle, char **out_error);

enum LrStatus lr_client_delete_by_category(struct LrClient *handle,
                                           const char *column,
                                           const char *value,
                                           char **out_error);

enum LrStatus lr_client_export(struct LrClient *handle,
                               const char *format,
                               const char *sort,
                               const char *direction,
                               uint8_t **out_data,
                               uintptr_t *out_len,
                               char **out_error);

enum LrStatus lr_client_input_user_export(struct LrClient *handle,
                                          const char *params_json,
                                          uint8_t **out_data,
                                          uintptr_t *out_len,
                                          char **out_error);

enum LrStatus lr_client_import_csv(struct LrClient *handle,
                                   const char *file_path,
                                   char **out_error);

enum LrStatus lr_config_debug_string(const char *base_url, char **out_str, char **out_error);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* LIBRCEKUNIT_H */
