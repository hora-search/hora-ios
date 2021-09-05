#include <stdint.h>

const char* rust_greeting(const char* to);
void rust_greeting_free(char *);
void index(const char* name, int dimension);
void add(const char* name, const float* features, int feature_size, int feature_idx);
void build(const char* name, const char* metrics_type);
const float* search(const char* name, int k, const float* features, int feature_size);
void load(const char* name, const char* file_path);
void dump(const char* name, const char* file_path);
