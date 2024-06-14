#include "../core/include/core.h"
#include "../core/include/macros.h"
#include "../core/include/wr_api.h"
#include "../langExampleCore/langExampleCore.h"
#include "external/quickjs/quickjs.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

const char *tagtos(long long tag) {
    switch (tag) {
    case JS_TAG_INT:
        return "int";
    case JS_TAG_BOOL:
        return "bool";
    case JS_TAG_NULL:
        return "null";
    case JS_TAG_UNDEFINED:
        return "undefined";
    case JS_TAG_CATCH_OFFSET:
        return "catch offset";
    case JS_TAG_EXCEPTION:
        return "exception";
    case JS_TAG_FLOAT64:
        return "float64";
    case JS_TAG_MODULE:
        return "module";
    case JS_TAG_OBJECT:
        return "object";
    case JS_TAG_STRING:
        return "string";
    case JS_TAG_FIRST:
        return "first";
    case JS_TAG_BIG_INT:
        return "big_int";
    case JS_TAG_BIG_FLOAT:
        return "big_float";
    case JS_TAG_SYMBOL:
        return "symbol";
    case JS_TAG_FUNCTION_BYTECODE:
        return "function bytecode";
    default:
        return "unknown type!";
    }
}

static JSContext *ctx;
static JSValue addLangExampleCallback;

void langExampleJS() {
    JSValue this = JS_DupValue(ctx, addLangExampleCallback);
    JSValue res = JS_Call(ctx, addLangExampleCallback, this, 0, NULL);
    if (JS_IsException(res)) {
        printf("JS err : %s\n", JS_ToCString(ctx, JS_GetException(ctx)));
    }
}

static JSValue jsAddLangExample(JSContext *ctx, JSValueConst thisVal, int argc,
                                JSValueConst *argv) {
    if (argc != 2) {
        return JS_EXCEPTION;
    }

    addLangExampleCallback = argv[1];
    // printf("addLangExampleCallback = %s : %s\n",
    //        JS_ToCString(ctx, addLangExampleCallback),
    //        tagtos(JS_VALUE_GET_TAG(addLangExampleCallback)));
    addLangExample(JS_ToCString(ctx, argv[0]), langExampleJS);
    return JS_UNDEFINED;
}

static JSValue jsLangExampleReturn(JSContext *ctx, JSValueConst thisVal,
                                   int argc, JSValueConst *argv) {
    if (argc != 1) {
        return JS_EXCEPTION;
    }
    langExampleReturn(JS_ToCString(ctx, argv[0]));
    return JS_UNDEFINED;
}

static const JSCFunctionListEntry jsLangExampleCoreFuncs[] = {
    JS_CFUNC_DEF("addLangExample", 1, jsAddLangExample),
    JS_CFUNC_DEF("langExampleReturn", 0, jsLangExampleReturn),
};

static int jsLangExampleCoreInit(JSContext *ctx, JSModuleDef *m) {
    return JS_SetModuleExportList(ctx, m, jsLangExampleCoreFuncs, 2);
}

JSModuleDef *jsInitModule(JSContext *ctx, const char *moduleName) {
    JSModuleDef *m;
    m = JS_NewCModule(ctx, moduleName, jsLangExampleCoreInit);
    if (!m)
        return NULL;
    JS_AddModuleExportList(ctx, m, jsLangExampleCoreFuncs, 2);
    return m;
}

WR_EXPORTED(void, init_mod_langExampleJS)() {
    const char *resName = "langExampleJS/wrres/example.js";
    int32_t rd = wr_res_open((WASMRawU64)resName, strlen(resName));
    int size = wr_res_get_size(rd);
    char *script = malloc(size + 1);
    wr_res_get_data(rd, (WASMRawU64)script);
    script[size] = '\0';
    wr_res_close(rd);

    // print_result[0]='\0';

    // printf( "if_contents: %s\n", script);

    // printf(
    //         "c0: %d\n", script[0]);

    JSRuntime *runtime = JS_NewRuntime();
    if (!runtime) {
        // printf( "line %d : JS_NewRuntime returned NULL\n", __LINE__-2);
        free(script);
        return;
    }
    ctx = JS_NewContext(runtime);
    if (!ctx) {
        // printf("line %d : JS_NewContext returned NULL\n", __LINE__-2);
        free(runtime);
        free(script);
        return;
    }
    JS_AddModuleExport(ctx, jsInitModule(ctx, "langExampleCore"),
                       "langExampleCore");
    JSValue result =
        JS_Eval(ctx, script, size, resName, JS_EVAL_TYPE_MODULE);
    if (JS_IsException(result)) {
        // printf("JS err : %s\n", JS_ToCString(ctx, JS_GetException(ctx)));
    } else if (JS_IsFunction(ctx, result)) {
        JSValue this = JS_DupValue(ctx, result);
        JSValue res = JS_Call(ctx, result, this, 0, NULL);
        // puts(JS_ToCString(ctx, result));
        // puts(JS_ToCString(ctx, res));
        JS_FreeValue(ctx, result);
        JS_FreeValue(ctx, res);
    } else {
        // printf("val = %s : %s\n", JS_ToCString(ctx, result),
        // tagtos(JS_VALUE_GET_TAG(result)));
    }
    JS_FreeValue(ctx, result);

    // JS_RunGC(runtime);

    // free(ctx);
    // free(runtime);
    // free(if_contents);
}
