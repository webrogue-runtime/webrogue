#pragma once

typedef void (*langExampleFunc)();

void addLangExample(const char *name, langExampleFunc func);

void langExampleReturn(const char *result);
