// verify.c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <openssl/evp.h>
#include <sys/stat.h>
#include <unistd.h>

#include "verify.h"

#define BUF_SIZE (128 * 1024 * 1024) // 4 MB buffer, similar to Rust

// Get file size (ISO)
static long get_file_size(const char *filename) {
    struct stat st;
    if (stat(filename, &st) != 0) return -1;
    return st.st_size;
}

// Compute SHA-256 hash of a file/device with limited size and print progress
int compute_sha256(const char *filename, unsigned char hash[EVP_MAX_MD_SIZE],
                   unsigned int *hash_len, long max_bytes) {
    FILE *file = fopen(filename, "rb");
    if (!file) {
        perror("fopen");
        return 0;
    }

    EVP_MD_CTX *mdctx = EVP_MD_CTX_new();
    if (!mdctx) {
        fclose(file);
        return 0;
    }

    if (EVP_DigestInit_ex(mdctx, EVP_sha256(), NULL) != 1) {
        EVP_MD_CTX_free(mdctx);
        fclose(file);
        return 0;
    }

    unsigned char *buffer = malloc(BUF_SIZE);
    if (!buffer) {
        perror("malloc");
        EVP_MD_CTX_free(mdctx);
        fclose(file);
        return 0;
    }

    long total_read = 0;
    size_t bytesRead;

    while (total_read < max_bytes &&
           (bytesRead = fread(buffer, 1, (size_t) ((max_bytes - total_read) > BUF_SIZE ? BUF_SIZE : (max_bytes - total_read)), file)) > 0) {

        if (EVP_DigestUpdate(mdctx, buffer, bytesRead) != 1) {
            free(buffer);
            EVP_MD_CTX_free(mdctx);
            fclose(file);
            return 0;
        }

        total_read += bytesRead;

        // Print percentage progress
        double percent = (double)total_read / (double)max_bytes * 100.0;
        printf("\rProgress: %6.2f%%", percent);
        fflush(stdout);
    }

    free(buffer);

    if (EVP_DigestFinal_ex(mdctx, hash, hash_len) != 1) {
        EVP_MD_CTX_free(mdctx);
        fclose(file);
        return 0;
    }

    EVP_MD_CTX_free(mdctx);
    fclose(file);

    printf("\rProgress: 100.00%%\n"); // ensure final progress
    return 1;
}

// Main verify function
bool verify(const char *iso_path, const char *dev_path) {

    #ifdef _WIN32
        system("cls");
    #else
        system("clear");
    #endif

    long iso_size = get_file_size(iso_path);
    if (iso_size <= 0) {
        fprintf(stderr, "Failed to get ISO size.\n");
        return false;
    }

    unsigned char iso_hash[EVP_MAX_MD_SIZE];
    unsigned char dev_hash[EVP_MAX_MD_SIZE];
    unsigned int len_iso, len_dev;

    printf("\nVerifying...\n");

    if (!compute_sha256(iso_path, iso_hash, &len_iso, iso_size) ||
        !compute_sha256(dev_path, dev_hash, &len_dev, iso_size)) {
        fprintf(stderr, "Error computing SHA-256 hash.\n");
        return false;
    }

    return (len_iso == len_dev && memcmp(iso_hash, dev_hash, len_iso) == 0);
}
