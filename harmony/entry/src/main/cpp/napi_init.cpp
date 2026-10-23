#include "napi/native_api.h"
#include <string>
extern "C" char* photoselect_scan(const char* path, bool recursive);
extern "C" char* photoselect_scan_paths_json(const char* root, const char* paths_json);
extern "C" void photoselect_free(char* value);
struct ScanTask { napi_async_work work = nullptr; napi_deferred deferred = nullptr; std::string path; std::string result; std::string file_list; bool recursive = false; bool use_file_list = false; };
static void ExecuteScan(napi_env, void* data) {
    auto* task = static_cast<ScanTask*>(data);
    char* result = task->use_file_list
        ? photoselect_scan_paths_json(task->path.c_str(), task->file_list.c_str())
        : photoselect_scan(task->path.c_str(), task->recursive);
    if (result == nullptr) { task->result = "{}"; return; }
    task->result.assign(result);
    photoselect_free(result);
}
static void CompleteScan(napi_env env, napi_status status, void* data) {
    auto* task = static_cast<ScanTask*>(data);
    napi_value value = nullptr;
    napi_create_string_utf8(env, task->result.c_str(), task->result.size(), &value);
    if (status == napi_ok) napi_resolve_deferred(env, task->deferred, value);
    else napi_reject_deferred(env, task->deferred, value);
    napi_delete_async_work(env, task->work);
    delete task;
}
static napi_value ScanDirectory(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value argv[2] = {nullptr, nullptr};
    napi_get_cb_info(env, info, &argc, argv, nullptr, nullptr);
    if (argc != 2) { napi_throw_type_error(env, nullptr, "scanDirectory expects path and recursive"); return nullptr; }
    size_t length = 0;
    if (napi_get_value_string_utf8(env, argv[0], nullptr, 0, &length) != napi_ok || length == 0) {
        napi_throw_type_error(env, nullptr, "path must be a non-empty string"); return nullptr;
    }
    bool recursive = false;
    if (napi_get_value_bool(env, argv[1], &recursive) != napi_ok) {
        napi_throw_type_error(env, nullptr, "recursive must be boolean"); return nullptr;
    }
    auto* task = new ScanTask();
    task->path.resize(length + 1);
    napi_get_value_string_utf8(env, argv[0], task->path.data(), length + 1, &length);
    task->path.resize(length);
    task->recursive = recursive;
    napi_value promise = nullptr;
    napi_value resource = nullptr;
    napi_create_promise(env, &task->deferred, &promise);
    napi_create_string_utf8(env, "PhotoSelect Rust directory scan", NAPI_AUTO_LENGTH, &resource);
    napi_status status = napi_create_async_work(env, nullptr, resource, ExecuteScan, CompleteScan, task, &task->work);
    if (status == napi_ok) status = napi_queue_async_work(env, task->work);
    if (status != napi_ok) {
        napi_value error = nullptr;
        napi_create_string_utf8(env, "Unable to queue Rust scan", NAPI_AUTO_LENGTH, &error);
        napi_reject_deferred(env, task->deferred, error);
        if (task->work != nullptr) napi_delete_async_work(env, task->work);
        delete task;
    }
    return promise;
}
static napi_value ScanAuthorizedFiles(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value argv[2] = {nullptr, nullptr};
    napi_get_cb_info(env, info, &argc, argv, nullptr, nullptr);
    if (argc != 2) { napi_throw_type_error(env, nullptr, "scanAuthorizedFiles expects root path and file list"); return nullptr; }
    size_t root_length = 0;
    size_t list_length = 0;
    if (napi_get_value_string_utf8(env, argv[0], nullptr, 0, &root_length) != napi_ok || root_length == 0 ||
        napi_get_value_string_utf8(env, argv[1], nullptr, 0, &list_length) != napi_ok || list_length == 0) {
        napi_throw_type_error(env, nullptr, "root path and file list must be non-empty strings"); return nullptr;
    }
    auto* task = new ScanTask();
    task->path.resize(root_length + 1);
    napi_get_value_string_utf8(env, argv[0], task->path.data(), root_length + 1, &root_length);
    task->path.resize(root_length);
    task->file_list.resize(list_length + 1);
    napi_get_value_string_utf8(env, argv[1], task->file_list.data(), list_length + 1, &list_length);
    task->file_list.resize(list_length);
    task->use_file_list = true;

    napi_value promise = nullptr;
    napi_value resource = nullptr;
    napi_create_promise(env, &task->deferred, &promise);
    napi_create_string_utf8(env, "PhotoSelect Rust authorized file scan", NAPI_AUTO_LENGTH, &resource);
    napi_status status = napi_create_async_work(env, nullptr, resource, ExecuteScan, CompleteScan, task, &task->work);
    if (status == napi_ok) status = napi_queue_async_work(env, task->work);
    if (status != napi_ok) {
        napi_value error = nullptr;
        napi_create_string_utf8(env, "Unable to queue Rust file scan", NAPI_AUTO_LENGTH, &error);
        napi_reject_deferred(env, task->deferred, error);
        if (task->work != nullptr) napi_delete_async_work(env, task->work);
        delete task;
    }
    return promise;
}
static napi_value Init(napi_env env, napi_value exports) {
    napi_property_descriptor properties[] = {
        {"scanDirectory", nullptr, ScanDirectory, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"scanAuthorizedFiles", nullptr, ScanAuthorizedFiles, nullptr, nullptr, nullptr, napi_default, nullptr}
    };
    napi_define_properties(env, exports, 2, properties);
    return exports;
}
static napi_module module = {1, 0, nullptr, Init, "photoselect", nullptr, {0}};
extern "C" __attribute__((constructor)) void RegisterPhotoSelectModule() { napi_module_register(&module); }
