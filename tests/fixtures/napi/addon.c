// SPDX-License-Identifier: MIT OR Apache-2.0
// Test addon for the N-API implementation.
// Build: cc -shared -fPIC -undefined dynamic_lookup -o addon.node addon.c  (macOS)
//        cc -shared -fPIC -o addon.node addon.c                           (Linux)
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <pthread.h>
#include <unistd.h>

typedef struct napi_env__ *napi_env;
typedef struct napi_value__ *napi_value;
typedef struct napi_callback_info__ *napi_callback_info;
typedef struct napi_async_cleanup_hook_handle__ *napi_async_cleanup_hook_handle;
typedef napi_value (*napi_callback)(napi_env, napi_callback_info);
typedef void (*napi_finalize)(napi_env, void *, void *);
typedef void (*napi_cleanup_hook)(void *);
typedef void (*napi_async_cleanup_hook)(napi_async_cleanup_hook_handle, void *);
typedef int napi_status;

extern napi_status napi_get_cb_info(napi_env, napi_callback_info, size_t *, napi_value *, napi_value *, void **);
extern napi_status napi_get_value_double(napi_env, napi_value, double *);
extern napi_status napi_get_value_string_utf8(napi_env, napi_value, char *, size_t, size_t *);
extern napi_status napi_create_double(napi_env, double, napi_value *);
extern napi_status napi_create_string_utf8(napi_env, const char *, size_t, napi_value *);
extern napi_status napi_coerce_to_string(napi_env, napi_value, napi_value *);
extern napi_status napi_coerce_to_object(napi_env, napi_value, napi_value *);
extern napi_status napi_create_function(napi_env, const char *, size_t, napi_callback, void *, napi_value *);
extern napi_status napi_set_named_property(napi_env, napi_value, const char *, napi_value);
extern napi_status napi_throw_error(napi_env, const char *, const char *);
extern napi_status napi_call_function(napi_env, napi_value, napi_value, size_t, const napi_value *, napi_value *);
extern napi_status napi_create_array(napi_env, napi_value *);
extern napi_status napi_set_element(napi_env, napi_value, unsigned, napi_value);
extern napi_status napi_create_object(napi_env, napi_value *);

typedef struct {
  const char *utf8name; napi_value name; napi_callback method; napi_callback getter;
  napi_callback setter; napi_value value; int attributes; void *data;
} napi_property_descriptor;
extern napi_status napi_define_class(napi_env, const char *, size_t, napi_callback, void *, size_t,
                                     const napi_property_descriptor *, napi_value *);
extern napi_status napi_define_properties(napi_env, napi_value, size_t, const napi_property_descriptor *);
extern napi_status napi_open_escapable_handle_scope(napi_env, void **);
extern napi_status napi_close_escapable_handle_scope(napi_env, void *);
extern napi_status napi_escape_handle(napi_env, void *, napi_value, napi_value *);

extern napi_status napi_has_property(napi_env, napi_value, napi_value, _Bool *);
extern napi_status napi_has_own_property(napi_env, napi_value, napi_value, _Bool *);
extern napi_status napi_delete_property(napi_env, napi_value, napi_value, _Bool *);
extern napi_status napi_object_freeze(napi_env, napi_value);
extern napi_status napi_object_seal(napi_env, napi_value);
extern napi_status napi_instanceof(napi_env, napi_value, napi_value, _Bool *);
extern napi_status napi_get_new_target(napi_env, napi_callback_info, napi_value *);
extern napi_status napi_set_instance_data(napi_env, void *, void *, void *);
extern napi_status napi_get_instance_data(napi_env, void **);
extern napi_status napi_create_arraybuffer(napi_env, size_t, void **, napi_value *);
extern napi_status napi_create_external_arraybuffer(napi_env, void *, size_t, napi_finalize, void *, napi_value *);
extern napi_status napi_get_arraybuffer_info(napi_env, napi_value, void **, size_t *);
extern napi_status napi_create_external_buffer(napi_env, size_t, void *, napi_finalize, void *, napi_value *);
extern napi_status napi_get_buffer_info(napi_env, napi_value, void **, size_t *);
extern napi_status napi_create_buffer(napi_env, size_t, void **, napi_value *);
extern napi_status napi_create_typedarray(napi_env, int, size_t, napi_value, size_t, napi_value *);
extern napi_status napi_create_dataview(napi_env, size_t, napi_value, size_t, napi_value *);
extern napi_status napi_get_dataview_info(napi_env, napi_value, size_t *, void **, napi_value *, size_t *);
extern napi_status napi_create_promise(napi_env, void **, napi_value *);
extern napi_status napi_resolve_deferred(napi_env, void *, napi_value);
extern napi_status napi_is_buffer(napi_env, napi_value, _Bool *);
extern napi_status napi_is_error(napi_env, napi_value, _Bool *);
extern napi_status napi_is_arraybuffer(napi_env, napi_value, _Bool *);
extern napi_status napi_is_promise(napi_env, napi_value, _Bool *);
extern napi_status napi_strict_equals(napi_env, napi_value, napi_value, _Bool *);
extern napi_status napi_get_boolean(napi_env, _Bool, napi_value *);
extern napi_status napi_create_symbol(napi_env, napi_value, napi_value *);
extern napi_status napi_coerce_to_number(napi_env, napi_value, napi_value *);
extern napi_status napi_coerce_to_bool(napi_env, napi_value, napi_value *);

extern napi_status napi_get_undefined(napi_env, napi_value *);
typedef struct napi_tsfn__ *napi_tsfn;
typedef void (*napi_tsfn_call_js)(napi_env, napi_value, void *, void *);
extern napi_status napi_create_threadsafe_function(napi_env, napi_value, napi_value, napi_value, size_t, size_t, void *, napi_finalize, void *, napi_tsfn_call_js, napi_tsfn *);
extern napi_status napi_call_threadsafe_function(napi_tsfn, void *, int);
extern napi_status napi_release_threadsafe_function(napi_tsfn, int);
extern napi_status napi_unref_threadsafe_function(napi_env, napi_tsfn);
extern napi_status napi_add_env_cleanup_hook(napi_env, napi_cleanup_hook, void *);
extern napi_status napi_remove_env_cleanup_hook(napi_env, napi_cleanup_hook, void *);
extern napi_status napi_add_async_cleanup_hook(napi_env, napi_async_cleanup_hook, void *, napi_async_cleanup_hook_handle *);
extern napi_status napi_remove_async_cleanup_hook(napi_async_cleanup_hook_handle);
extern napi_status napi_create_bigint_int64(napi_env, int64_t, napi_value *);
extern napi_status napi_create_bigint_uint64(napi_env, uint64_t, napi_value *);
extern napi_status napi_create_bigint_words(napi_env, int, size_t, const uint64_t *, napi_value *);
extern napi_status napi_get_value_bigint_int64(napi_env, napi_value, int64_t *, _Bool *);
extern napi_status napi_get_value_bigint_uint64(napi_env, napi_value, uint64_t *, _Bool *);
extern napi_status napi_get_value_bigint_words(napi_env, napi_value, int *, size_t *, uint64_t *);
extern napi_status napi_create_date(napi_env, double, napi_value *);
extern napi_status napi_get_date_value(napi_env, napi_value, double *);
extern napi_status napi_create_external(napi_env, void *, napi_finalize, void *, napi_value *);
extern napi_status napi_get_value_external(napi_env, napi_value, void **);
extern napi_status napi_typeof(napi_env, napi_value, int *);
extern napi_status napi_create_string_latin1(napi_env, const char *, size_t, napi_value *);
extern napi_status napi_get_value_string_utf16(napi_env, napi_value, uint16_t *, size_t, size_t *);
extern napi_status napi_run_script(napi_env, napi_value, napi_value *);
extern napi_status napi_get_property_names(napi_env, napi_value, napi_value *);
extern napi_status napi_get_all_property_names(napi_env, napi_value, int, int, int, napi_value *);

static void tsfn_call_js(napi_env env, napi_value cb, void *context, void *data) {
  napi_value arg, recv, result;
  napi_create_double(env, (double)(intptr_t)data, &arg);
  napi_get_undefined(env, &recv);
  napi_call_function(env, recv, cb, 1, &arg, &result);
}

static void *tsfn_thread(void *p) {
  napi_tsfn f = (napi_tsfn)p;
  for (intptr_t i = 1; i <= 3; i++) {
    usleep(10000);
    napi_call_threadsafe_function(f, (void *)i, 0);
  }
  napi_release_threadsafe_function(f, 0);
  return NULL;
}

static int tsfn_finalize_count;
static void cleanup_hook(void *arg) { (void)arg; }
static void async_cleanup_hook(napi_async_cleanup_hook_handle handle, void *arg) {
  (void)handle;
  (void)arg;
}

static void tsfn_finalize(napi_env env, void *data, void *hint) {
  (void)env;
  (void)data;
  (void)hint;
  tsfn_finalize_count++;
}

static napi_value start_thread(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1];
  napi_tsfn f;
  pthread_t th;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_create_threadsafe_function(env, argv[0], NULL, NULL, 0, 1, NULL, tsfn_finalize, NULL, tsfn_call_js, &f);
  napi_unref_threadsafe_function(env, f);
  pthread_create(&th, NULL, tsfn_thread, f);
  pthread_detach(th);
  return NULL;
}

static napi_value tsfn_finalize_count_value(napi_env env, napi_callback_info info) {
  (void)info;
  napi_value result;
  napi_create_double(env, (double)tsfn_finalize_count, &result);
  return result;
}

static napi_value is_error_value(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1], result;
  _Bool is_error = false;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_is_error(env, argv[0], &is_error);
  napi_get_boolean(env, is_error, &result);
  return result;
}

static napi_value is_buffer_value(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1], result;
  _Bool is_buffer = false;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_is_buffer(env, argv[0], &is_buffer);
  napi_get_boolean(env, is_buffer, &result);
  return result;
}

static napi_value create_buffer_value(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1], result;
  double size = 0;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  if (argc > 0) napi_get_value_double(env, argv[0], &size);
  napi_create_buffer(env, (size_t)size, NULL, &result);
  return result;
}

static napi_value buffer_info_status(napi_env env, napi_callback_info info) {
  size_t argc = 1, length = 0;
  napi_value argv[1], result;
  void *data = NULL;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_status status = argc > 0 ? napi_get_buffer_info(env, argv[0], &data, &length) : 1;
  napi_create_double(env, (double)status, &result);
  return result;
}

static napi_value cleanup_hook_statuses(napi_env env, napi_callback_info info) {
  (void)info;
  napi_status statuses[4];
  napi_async_cleanup_hook_handle handle = NULL;
  statuses[0] = napi_add_env_cleanup_hook(env, cleanup_hook, NULL);
  statuses[1] = napi_remove_env_cleanup_hook(env, cleanup_hook, NULL);
  statuses[2] = napi_add_async_cleanup_hook(env, async_cleanup_hook, NULL, &handle);
  statuses[3] = napi_remove_async_cleanup_hook(handle);

  napi_value result;
  napi_create_array(env, &result);
  for (unsigned i = 0; i < sizeof(statuses) / sizeof(statuses[0]); i++) {
    napi_value status;
    napi_create_double(env, (double)statuses[i], &status);
    napi_set_element(env, result, i, status);
  }
  return result;
}

static uint8_t external_arraybuffer_data[] = {1, 2, 3, 4};
static uint8_t external_buffer_data[] = {5, 6, 7, 8};
static int external_finalize_count = 0;

static void count_external_finalize(napi_env env, void *data, void *hint) {
  (void)env;
  (void)data;
  (void)hint;
  external_finalize_count++;
}

static napi_value external_arraybuffer(napi_env env, napi_callback_info info) {
  napi_value result;
  napi_create_external_arraybuffer(env, external_arraybuffer_data, sizeof(external_arraybuffer_data),
                                  count_external_finalize, NULL, &result);
  return result;
}

static napi_value external_arraybuffer_value(napi_env env, napi_callback_info info) {
  napi_value result;
  napi_create_double(env, external_arraybuffer_data[0], &result);
  return result;
}

static napi_value external_buffer(napi_env env, napi_callback_info info) {
  napi_value result;
  napi_create_external_buffer(env, sizeof(external_buffer_data), external_buffer_data,
                              count_external_finalize, NULL, &result);
  return result;
}

static napi_value external_buffer_value(napi_env env, napi_callback_info info) {
  napi_value result;
  napi_create_double(env, external_buffer_data[0], &result);
  return result;
}

static napi_value external_finalize_count_value(napi_env env, napi_callback_info info) {
  napi_value result;
  napi_create_double(env, external_finalize_count, &result);
  return result;
}

static napi_value sample_constructor(napi_env env, napi_callback_info info) {
  napi_value new_target;
  napi_get_new_target(env, info, &new_target);
  return NULL;
}

static napi_value bigints(napi_env env, napi_callback_info info) {
  size_t argc = 1, count = 4;
  napi_value argv[1], arr, a, b, u;
  int64_t i64; uint64_t u64; _Bool lossless, ulossless; int sign; uint64_t words[4];
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_get_value_bigint_int64(env, argv[0], &i64, &lossless);
  napi_get_value_bigint_uint64(env, argv[0], &u64, &ulossless);
  napi_get_value_bigint_words(env, argv[0], &sign, &count, words);
  uint64_t big[2] = {1, 2};
  napi_create_bigint_words(env, 1, 2, big, &a);
  napi_create_bigint_int64(env, i64 * 2, &b);
  napi_create_bigint_uint64(env, u64, &u);
  napi_create_array(env, &arr);
  napi_set_element(env, arr, 0, a);
  napi_set_element(env, arr, 1, b);
  napi_set_element(env, arr, 6, u);
  napi_value n, l, s, w0, ul;
  napi_create_double(env, (double)count, &n);
  napi_get_boolean(env, lossless, &l);
  napi_get_boolean(env, ulossless, &ul);
  napi_create_double(env, (double)sign, &s);
  napi_create_double(env, (double)words[0], &w0);
  napi_set_element(env, arr, 2, n); napi_set_element(env, arr, 3, l);
  napi_set_element(env, arr, 4, s); napi_set_element(env, arr, 5, w0);
  napi_set_element(env, arr, 7, ul);
  return arr;
}

static napi_value misc(napi_env env, napi_callback_info info) {
  napi_value arr, d, ext, ty, latin, script, sres, u16;
  double t = 0; int type; void *p = NULL; size_t n = 0; uint16_t buf[8];
  static int payload = 42;
  napi_create_array(env, &arr);
  napi_create_date(env, 86400000.0, &d);
  napi_set_element(env, arr, 0, d);
  napi_get_date_value(env, d, &t);
  napi_create_double(env, t, &ty); napi_set_element(env, arr, 1, ty);
  napi_create_external(env, &payload, NULL, NULL, &ext);
  napi_typeof(env, ext, &type);
  napi_get_value_external(env, ext, &p);
  napi_create_double(env, type * 1000 + *(int *)p, &ty); napi_set_element(env, arr, 2, ty);
  napi_create_string_latin1(env, "caf\xe9", (size_t)-1, &latin);
  napi_set_element(env, arr, 3, latin);
  napi_get_value_string_utf16(env, latin, buf, 8, &n);
  napi_create_double(env, (double)(n * 1000 + buf[3]), &u16); napi_set_element(env, arr, 4, u16);
  napi_create_string_utf8(env, "1 + 2 * 3", (size_t)-1, &script);
  napi_run_script(env, script, &sres); napi_set_element(env, arr, 5, sres);
  return arr;
}

static napi_value all_keys(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1], r;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_get_all_property_names(env, argv[0], 0, 2 | 16, 0, &r);
  return r;
}

static napi_value property_names(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1], r;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_get_property_names(env, argv[0], &r);
  return r;
}

static napi_value add(napi_env env, napi_callback_info info) {
  size_t argc = 2;
  napi_value argv[2];
  double a, b, r;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  if (napi_get_value_double(env, argv[0], &a) != 0 || napi_get_value_double(env, argv[1], &b) != 0) {
    napi_throw_error(env, "ERR_INVALID_ARG", "Expected two numbers");
    return NULL;
  }
  r = a + b;
  napi_value result;
  napi_create_double(env, r, &result);
  return result;
}

static napi_value greet(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1];
  char name[64];
  char out[96] = "Hello, ";
  size_t len = 0, i = 7;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_get_value_string_utf8(env, argv[0], name, sizeof(name), &len);
  for (size_t j = 0; j < len && i < sizeof(out) - 2; j++) out[i++] = name[j];
  out[i++] = '!';
  napi_value result;
  napi_create_string_utf8(env, out, i, &result);
  return result;
}

static napi_value call_twice(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1], recv, result;
  napi_get_cb_info(env, info, &argc, argv, &recv, NULL);
  napi_call_function(env, recv, argv[0], 0, NULL, &result);
  return result;
}

static napi_value escaped(napi_env env, napi_callback_info info) {
  void *scope;
  napi_value tmp, result;
  napi_open_escapable_handle_scope(env, &scope);
  napi_create_double(env, 7, &tmp);
  napi_escape_handle(env, scope, tmp, &result);
  napi_close_escapable_handle_scope(env, scope);
  return result;
}

static napi_value has_del(napi_env env, napi_callback_info info) {
  size_t argc = 2;
  napi_value argv[2], r;
  _Bool has, own, del;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_has_property(env, argv[0], argv[1], &has);
  napi_has_own_property(env, argv[0], argv[1], &own);
  napi_delete_property(env, argv[0], argv[1], &del);
  napi_create_double(env, (has ? 1 : 0) + (del ? 10 : 0) + (own ? 100 : 0), &r);
  return r;
}

static napi_value freeze_object(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1];
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_object_freeze(env, argv[0]);
  return argv[0];
}

static napi_value seal_object(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1];
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_object_seal(env, argv[0]);
  return argv[0];
}

static napi_value is_instance(napi_env env, napi_callback_info info) {
  size_t argc = 2;
  napi_value argv[2], r;
  _Bool b = 0;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_instanceof(env, argv[0], argv[1], &b);
  napi_get_boolean(env, b, &r);
  return r;
}

static napi_value new_target(napi_env env, napi_callback_info info) {
  napi_value nt, r;
  napi_get_new_target(env, info, &nt);
  napi_get_boolean(env, nt != NULL, &r);
  return r;
}

static napi_value counter(napi_env env, napi_callback_info info) {
  void *p = NULL;
  napi_value r;
  napi_get_instance_data(env, &p);
  size_t n = (size_t)p + 1;
  napi_set_instance_data(env, (void *)n, NULL, NULL);
  napi_create_double(env, (double)n, &r);
  return r;
}

/* arrayBuffer(n) -> ArrayBuffer filled with 0..n-1; views(ab) -> [Uint16Array(ab), DataView(ab), sum] */
static napi_value array_buffer(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1], ab;
  double n;
  void *data;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_get_value_double(env, argv[0], &n);
  napi_create_arraybuffer(env, (size_t)n, &data, &ab);
  for (size_t i = 0; i < (size_t)n; i++) ((unsigned char *)data)[i] = (unsigned char)i;
  return ab;
}

static napi_value views(napi_env env, napi_callback_info info) {
  size_t argc = 1, len, dvlen, dvoff;
  napi_value argv[1], u16, dv, arr, tmp, dvab;
  void *data, *dvdata;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_get_arraybuffer_info(env, argv[0], &data, &len);
  napi_create_typedarray(env, 4, len / 2, argv[0], 0, &u16);
  napi_create_dataview(env, len, argv[0], 0, &dv);
  napi_get_dataview_info(env, dv, &dvlen, &dvdata, &dvab, &dvoff);
  double sum = 0;
  for (size_t i = 0; i < len; i++) sum += ((unsigned char *)data)[i];
  napi_create_array(env, &arr);
  napi_set_element(env, arr, 0, u16);
  napi_set_element(env, arr, 1, dv);
  napi_create_double(env, sum + (dvdata == data ? 1000 : 0) + dvlen * 10000, &tmp);
  napi_set_element(env, arr, 2, tmp);
  return arr;
}

static napi_value coerce(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1], n, b, arr, sym, d, string, object;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_coerce_to_number(env, argv[0], &n);
  napi_coerce_to_bool(env, argv[0], &b);
  napi_coerce_to_string(env, argv[0], &string);
  napi_coerce_to_object(env, argv[0], &object);
  napi_create_string_utf8(env, "desc", (size_t)-1, &d);
  napi_create_symbol(env, d, &sym);
  napi_create_array(env, &arr);
  napi_set_element(env, arr, 0, n);
  napi_set_element(env, arr, 1, b);
  napi_set_element(env, arr, 2, sym);
  napi_set_element(env, arr, 3, string);
  napi_set_element(env, arr, 4, object);
  return arr;
}

static napi_value strict_equals(napi_env env, napi_callback_info info) {
  size_t argc = 2;
  napi_value argv[2], result;
  _Bool equal;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_strict_equals(env, argv[0], argv[1], &equal);
  napi_get_boolean(env, equal, &result);
  return result;
}

static napi_value is_promise(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1], result;
  _Bool is_promise_value;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_is_promise(env, argv[0], &is_promise_value);
  napi_get_boolean(env, is_promise_value, &result);
  return result;
}

static napi_value is_arraybuffer(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1], result;
  _Bool is_arraybuffer_value;
  napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
  napi_is_arraybuffer(env, argv[0], &is_arraybuffer_value);
  napi_get_boolean(env, is_arraybuffer_value, &result);
  return result;
}

static napi_value resolved_promise(napi_env env, napi_callback_info info) {
  void *deferred;
  napi_value promise, value;
  napi_create_promise(env, &deferred, &promise);
  napi_create_string_utf8(env, "native promise", (size_t)-1, &value);
  napi_resolve_deferred(env, deferred, value);
  return promise;
}

napi_value napi_register_module_v1(napi_env env, napi_value exports) {
  napi_value fn;
  napi_create_function(env, "externalArrayBuffer", (size_t)-1, external_arraybuffer, NULL, &fn);
  napi_set_named_property(env, exports, "externalArrayBuffer", fn);
  napi_create_function(env, "externalArrayBufferValue", (size_t)-1, external_arraybuffer_value, NULL, &fn);
  napi_set_named_property(env, exports, "externalArrayBufferValue", fn);
  napi_create_function(env, "externalBuffer", (size_t)-1, external_buffer, NULL, &fn);
  napi_set_named_property(env, exports, "externalBuffer", fn);
  napi_create_function(env, "externalBufferValue", (size_t)-1, external_buffer_value, NULL, &fn);
  napi_set_named_property(env, exports, "externalBufferValue", fn);
  napi_create_function(env, "externalFinalizeCount", (size_t)-1, external_finalize_count_value, NULL, &fn);
  napi_set_named_property(env, exports, "externalFinalizeCount", fn);
  napi_define_class(env, "Sample", (size_t)-1, sample_constructor, NULL, 0, NULL, &fn);
  napi_set_named_property(env, exports, "Sample", fn);
  napi_create_function(env, "add", (size_t)-1, add, NULL, &fn);
  napi_set_named_property(env, exports, "add", fn);
  napi_create_function(env, "greet", (size_t)-1, greet, NULL, &fn);
  napi_set_named_property(env, exports, "greet", fn);
  napi_create_function(env, "callTwice", (size_t)-1, call_twice, NULL, &fn);
  napi_set_named_property(env, exports, "callTwice", fn);
  napi_create_function(env, "hasDel", (size_t)-1, has_del, NULL, &fn);
  napi_set_named_property(env, exports, "hasDel", fn);
  napi_create_function(env, "freezeObject", (size_t)-1, freeze_object, NULL, &fn);
  napi_set_named_property(env, exports, "freezeObject", fn);
  napi_create_function(env, "sealObject", (size_t)-1, seal_object, NULL, &fn);
  napi_set_named_property(env, exports, "sealObject", fn);
  napi_create_function(env, "isInstance", (size_t)-1, is_instance, NULL, &fn);
  napi_set_named_property(env, exports, "isInstance", fn);
  napi_create_function(env, "newTarget", (size_t)-1, new_target, NULL, &fn);
  napi_set_named_property(env, exports, "newTarget", fn);
  napi_create_function(env, "counter", (size_t)-1, counter, NULL, &fn);
  napi_set_named_property(env, exports, "counter", fn);
  napi_create_function(env, "arrayBuffer", (size_t)-1, array_buffer, NULL, &fn);
  napi_set_named_property(env, exports, "arrayBuffer", fn);
  napi_create_function(env, "views", (size_t)-1, views, NULL, &fn);
  napi_set_named_property(env, exports, "views", fn);
  napi_create_function(env, "coerce", (size_t)-1, coerce, NULL, &fn);
  napi_set_named_property(env, exports, "coerce", fn);
  napi_create_function(env, "strictEquals", (size_t)-1, strict_equals, NULL, &fn);
  napi_set_named_property(env, exports, "strictEquals", fn);
  napi_create_function(env, "isPromise", (size_t)-1, is_promise, NULL, &fn);
  napi_set_named_property(env, exports, "isPromise", fn);
  napi_create_function(env, "isArrayBuffer", (size_t)-1, is_arraybuffer, NULL, &fn);
  napi_set_named_property(env, exports, "isArrayBuffer", fn);
  napi_create_function(env, "resolvedPromise", (size_t)-1, resolved_promise, NULL, &fn);
  napi_set_named_property(env, exports, "resolvedPromise", fn);
  napi_create_function(env, "startThread", (size_t)-1, start_thread, NULL, &fn);
  napi_set_named_property(env, exports, "startThread", fn);
  napi_create_function(env, "tsfnFinalizeCount", (size_t)-1, tsfn_finalize_count_value, NULL, &fn);
  napi_set_named_property(env, exports, "tsfnFinalizeCount", fn);
  napi_create_function(env, "isError", (size_t)-1, is_error_value, NULL, &fn);
  napi_set_named_property(env, exports, "isError", fn);
  napi_create_function(env, "isBuffer", (size_t)-1, is_buffer_value, NULL, &fn);
  napi_set_named_property(env, exports, "isBuffer", fn);
  napi_create_function(env, "createBuffer", (size_t)-1, create_buffer_value, NULL, &fn);
  napi_set_named_property(env, exports, "createBuffer", fn);
  napi_create_function(env, "bufferInfoStatus", (size_t)-1, buffer_info_status, NULL, &fn);
  napi_set_named_property(env, exports, "bufferInfoStatus", fn);
  napi_create_function(env, "cleanupHookStatuses", (size_t)-1, cleanup_hook_statuses, NULL, &fn);
  napi_set_named_property(env, exports, "cleanupHookStatuses", fn);
  napi_create_function(env, "bigints", (size_t)-1, bigints, NULL, &fn);
  napi_set_named_property(env, exports, "bigints", fn);
  napi_create_function(env, "misc", (size_t)-1, misc, NULL, &fn);
  napi_set_named_property(env, exports, "misc", fn);
  napi_create_function(env, "allKeys", (size_t)-1, all_keys, NULL, &fn);
  napi_set_named_property(env, exports, "allKeys", fn);
  napi_create_function(env, "propertyNames", (size_t)-1, property_names, NULL, &fn);
  napi_set_named_property(env, exports, "propertyNames", fn);
  napi_value seven;
  napi_create_double(env, 7, &seven);
  napi_property_descriptor props[] = {
    {"hidden", NULL, NULL, NULL, NULL, seven, 0, NULL},
    {"visible", NULL, NULL, NULL, NULL, seven, 2 | 1, NULL},
    {"escaped", NULL, escaped, NULL, NULL, NULL, 2, NULL},
  };
  napi_define_properties(env, exports, 3, props);
  return exports;
}
