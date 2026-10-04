#include "librcekunit.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ------------------------------------------------------------------------- */
/* Helper                                                                    */
/* ------------------------------------------------------------------------- */

static const char *status_name(LrStatus st) {
  switch (st) {
  case LR_STATUS_OK:
    return "OK";
  case LR_STATUS_INVALID_ARG:
    return "INVALID_ARG";
  case LR_STATUS_CONFIG:
    return "CONFIG";
  case LR_STATUS_AUTH:
    return "AUTH";
  case LR_STATUS_NETWORK:
    return "NETWORK";
  case LR_STATUS_API:
    return "API";
  case LR_STATUS_IO:
    return "IO";
  case LR_STATUS_INTERNAL:
    return "INTERNAL";
  case LR_STATUS_PANIC:
    return "PANIC";
  default:
    return "UNKNOWN";
  }
}

static void dump_preview(const char *label, const char *body) {
  if (!body) {
    printf("  [%s] (body NULL)\n", label);
    return;
  }
  size_t n = strlen(body);
  size_t preview = n < 400 ? n : 400;
  printf("  [%s] %zu bytes, preview:\n---\n", label, n);
  fwrite(body, 1, preview, stdout);
  if (n > preview)
    printf("\n... (truncated, %zu more bytes)", n - preview);
  printf("\n---\n");
}

/* Kembalikan 0 kalau OK, 1 kalau error. Tetap print body kalau ada. */
static int check(LrStatus st, char *err, const char *what, const char *body) {
  if (st == LR_STATUS_OK)
    return 0;
  fprintf(stderr, "[%s] status=%d (%s) error=%s\n", what, (int)st,
          status_name(st), err ? err : "(none)");
  if (body)
    dump_preview(what, body);
  if (err)
    lr_string_free(err);
  return 1;
}

/* ------------------------------------------------------------------------- */
/* Main                                                                      */
/* ------------------------------------------------------------------------- */

int main(int argc, char **argv) {
  const char *base_url = argc > 1 ? argv[1] : "http://103.31.38.200";
  const char *email = argc > 2 ? argv[2] : NULL;
  const char *password = argc > 3 ? argv[3] : NULL;

  if (!email || !password) {
    fprintf(stderr, "usage: %s <base_url> <email> <password>\n", argv[0]);
    fprintf(stderr,
            "       (kredensial wajib, contoh di source cuma placeholder)\n");
    return 2;
  }

  LrClient *client = NULL;
  char *err = NULL;

  /* ------------------------------------------------------------------ */
  /* 1. Create client                                                    */
  /* ------------------------------------------------------------------ */
  printf("[1] lr_client_new(%s)\n", base_url);
  LrStatus st = lr_client_new(base_url, "session.json", /* persistent cookie */
                              "c-consumer/1.0", 1800,   /* timeout 30 menit */
                              &client, &err);
  if (check(st, err, "new", NULL))
    return 1;
  puts("    [ok] client created");

  /* ------------------------------------------------------------------ */
  /* 2. Login                                                            */
  /* ------------------------------------------------------------------ */
  printf("[2] lr_client_login(%s)\n", email);
  st = lr_client_login(client, email, password, &err);
  if (st != LR_STATUS_OK) {
    check(st, err, "login", NULL);
    lr_client_free(client);
    return 1;
  }
  puts("    [ok] login request success");

  /* ------------------------------------------------------------------ */
  /* 3. Index вЂ” print body apapun hasilnya                               */
  /* ------------------------------------------------------------------ */
  printf("[3] lr_client_index\n");
  char *body = NULL;
  st = lr_client_index(client, &body, &err);
  if (st != LR_STATUS_OK) {
    check(st, err, "index", body);
    if (body)
      lr_string_free(body);
    lr_client_free(client);
    return 1;
  }
  printf("    [ok] index: %zu bytes\n", body ? strlen(body) : 0);
  if (body)
    lr_string_free(body);

  /* ------------------------------------------------------------------ */
  /* 4. Store вЂ” kirim JSON form                                          */
  /* ------------------------------------------------------------------ */
  printf("[4] lr_client_store\n");
  const char *form = "{\"no_perjanjian\":\"SP-25-F0001843\","
                     "\"nama_nasabah\":\"SADIMAN\","
                     "\"nopol\":\"BP2927GU\"}";

  body = NULL;
  st = lr_client_store(client, form, &body, &err);
  if (st != LR_STATUS_OK) {
    check(st, err, "store", body);
    if (body)
      lr_string_free(body);
    lr_client_free(client);
    return 1;
  }
  printf("    [ok] store: %zu bytes\n", body ? strlen(body) : 0);
  if (body)
    lr_string_free(body);

  /* ------------------------------------------------------------------ */
  /* 5. Export CSV вЂ” binary response                                     */
  /* ------------------------------------------------------------------ */
  printf("[5] lr_client_export(csv)\n");
  unsigned char *data = NULL;
  size_t data_len = 0;
  st = lr_client_export(client, "csv", "nomor", "asc", &data, &data_len, &err);
  if (st != LR_STATUS_OK) {
    check(st, err, "export", NULL);
    lr_client_free(client);
    return 1;
  }
  printf("    [ok] export: %zu bytes\n", data_len);

  FILE *f = fopen("export.csv", "wb");
  if (f) {
    fwrite(data, 1, data_len, f);
    fclose(f);
    puts("    [ok] wrote export.csv");
  } else {
    fprintf(stderr, "    [!] could not open export.csv for write\n");
  }
  lr_bytes_free(data, data_len);

  /* ------------------------------------------------------------------ */
  /* 6. Import CSV (optional)                                            */
  /* ------------------------------------------------------------------ */
  printf("[6] lr_client_import_csv(input_user.csv)\n");
  st = lr_client_import_csv(client, "input_user.csv", &err);
  if (st != LR_STATUS_OK) {
    check(st, err, "import_csv", NULL);
    /* jangan abort; file mungkin tidak ada */
  } else {
    puts("    [ok] imported input_user.csv");
  }

  /* ------------------------------------------------------------------ */
  /* 7. Logout                                                           */
  /* ------------------------------------------------------------------ */
  printf("[7] lr_client_logout\n");
  st = lr_client_logout(client, &err);
  if (st != LR_STATUS_OK) {
    check(st, err, "logout", NULL);
    lr_client_free(client);
    return 1;
  }
  puts("    [ok] logged out");

  lr_client_free(client);
  puts("[done]");
  return 0;
}
