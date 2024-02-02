#pragma once

typedef const char *(*langExampleFunc)();

void addLangExample(const char *name, langExampleFunc func);
