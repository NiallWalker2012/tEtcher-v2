#include <stdbool.h>

#ifndef VERIFY_H
#define VERIFY_H

static long get_file_size(const char *filename);
int compute_sha256(const char *filename, unsigned char hash[EVP_MAX_MD_SIZE], unsigned int *hash_len, long max_bytes);
bool verify(const char *iso_path, const char *dev_path);

#endif