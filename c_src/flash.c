#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>
#include <errno.h>
#include <string.h>
#include <stdbool.h>

#define BUFFER_SIZE (128 * 1024 * 1024)
#define ALIGNMENT   4096

void flash(const char *iso_path, const char *dev_path) {
    #ifdef __WIN32
        system("cls");
    #else
        system("clear");
    #endif

    // -------------------------------
    // Open files
    // -------------------------------
    int fd_iso = open(iso_path, O_RDONLY);
    int fd_dev = open(dev_path, O_WRONLY);
    if (fd_iso < 0 || fd_dev < 0) {
        perror("open");
        if (fd_iso >= 0) close(fd_iso);
        if (fd_dev >= 0) close(fd_dev);
        exit(EXIT_FAILURE);
    }

    // -------------------------------
    // Allocate aligned buffer for O_DIRECT
    // -------------------------------
    void *buffer = NULL;
    if (posix_memalign(&buffer, ALIGNMENT, BUFFER_SIZE) != 0) {
        fprintf(stderr, "Failed to allocate aligned buffer\n");
        close(fd_iso);
        close(fd_dev);
        exit(EXIT_FAILURE);
    }

    // -------------------------------
    // Determine ISO size for progress calculation
    // -------------------------------
    struct stat st;
    if (fstat(fd_iso, &st) != 0) {
        perror("fstat");
        free(buffer);
        close(fd_iso);
        close(fd_dev);
        exit(EXIT_FAILURE);
    }
    off_t iso_size = st.st_size;

    off_t total_copied = 0;
    ssize_t read_bytes, written_bytes;
    int last_percent = -1;

    const char *bar = "##################################################"; // 50 chars for the bar
    const int bar_width = 50;

    // -------------------------------
    // Copy loop
    // -------------------------------
    while ((read_bytes = read(fd_iso, buffer, BUFFER_SIZE)) > 0) {
        ssize_t total_written = 0;

        // Write entire buffer to device
        while (total_written < read_bytes) {
            written_bytes = write(fd_dev, (char *)buffer + total_written, read_bytes - total_written);
            if (written_bytes < 0) {
                perror("write");
                free(buffer);
                close(fd_iso);
                close(fd_dev);
                exit(EXIT_FAILURE);
            }
            total_written += written_bytes;
        }

        total_copied += read_bytes;

        // -------------------------------
        // Progress display with bar
        // -------------------------------
        int percent = (int)((total_copied * 100LL) / iso_size);
        if (percent != last_percent) {
            int progress_chars = (percent * bar_width) / 100;
            fprintf(stderr, "\rProgress: [%-*.*s] %3d%%", bar_width, progress_chars, bar, percent);
            fflush(stderr);
            last_percent = percent;
        }
    }

    if (read_bytes < 0)
        perror("read");

    // -------------------------------
    // Flush to device BEFORE returning
    // -------------------------------
    fprintf(stderr, "\nFlushing data to disk... (this may take a while)\n");
    fflush(stderr);

    if (fsync(fd_dev) != 0)
        perror("fsync");

    // -------------------------------
    // Final message after flush completes
    // -------------------------------
    fprintf(stderr, "Finished flashing!\n");
    fflush(stderr);

    // -------------------------------
    // Cleanup
    // -------------------------------
    close(fd_dev);
    close(fd_iso);
    free(buffer);
}

