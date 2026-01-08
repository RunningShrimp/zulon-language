// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// ZULON Runtime - Entry Point and I/O Functions
//
// This file provides the C runtime entry point for ZULON programs
// and basic I/O operations.

#ifndef _WIN32
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#else
#include <windows.h>
#include <io.h>
#endif

// ZULON main function (defined in user code)
extern int zulon_main(void);

// ZULON runtime entry point
#ifdef _WIN32
int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    return zulon_main();
}
#else
int main(int argc, char** argv) {
    // Ignore command-line arguments for now
    (void)argc;
    (void)argv;

    // Call ZULON main function
    int result = zulon_main();

    // Exit with result code
    exit(result);
}
#endif

// ============================================================================
// I/O Functions
// ============================================================================

// Print a single character
void zulon_putchar(char c) {
#ifdef _WIN32
    HANDLE hStdOut = GetStdHandle(STD_OUTPUT_HANDLE);
    DWORD written;
    WriteFile(hStdOut, &c, 1, &written, NULL);
#else
    write(STDOUT_FILENO, &c, 1);
#endif
}

// Print a null-terminated string
void zulon_print(const char* str) {
    if (str == NULL) {
        return;
    }

#ifdef _WIN32
    HANDLE hStdOut = GetStdHandle(STD_OUTPUT_HANDLE);
    DWORD len = (DWORD)strlen(str);
    DWORD written;
    WriteFile(hStdOut, str, len, &written, NULL);
#else
    size_t len = strlen(str);
    write(STDOUT_FILENO, str, len);
#endif
}

// Print a null-terminated string with newline
void zulon_println(const char* str) {
    zulon_print(str);
    zulon_putchar('\n');
}

// Print an integer as decimal
void zulon_print_i32(int32_t value) {
    char buffer[32];
    int i = 0;
    int negative = 0;

    if (value < 0) {
        negative = 1;
        value = -value;
    }

    // Handle zero
    if (value == 0) {
        zulon_putchar('0');
        return;
    }

    // Convert to string (reverse order)
    while (value > 0) {
        buffer[i++] = '0' + (value % 10);
        value /= 10;
    }

    // Add negative sign
    if (negative) {
        zulon_putchar('-');
    }

    // Print in reverse order
    while (i > 0) {
        zulon_putchar(buffer[--i]);
    }
}

// Print an integer as decimal with newline
void zulon_println_i32(int32_t value) {
    zulon_print_i32(value);
    zulon_putchar('\n');
}

// Print a 64-bit integer
void zulon_print_i64(int64_t value) {
    char buffer[64];
    int i = 0;
    int negative = 0;

    if (value < 0) {
        negative = 1;
        value = -value;
    }

    if (value == 0) {
        zulon_putchar('0');
        return;
    }

    while (value > 0) {
        buffer[i++] = '0' + (value % 10);
        value /= 10;
    }

    if (negative) {
        zulon_putchar('-');
    }

    while (i > 0) {
        zulon_putchar(buffer[--i]);
    }
}

// Print a 64-bit integer with newline
void zulon_println_i64(int64_t value) {
    zulon_print_i64(value);
    zulon_putchar('\n');
}

// Print a floating-point number (simplified)
void zulon_print_f64(double value) {
#ifdef _WIN32
    // Windows uses different format specifier
    char buffer[128];
    sprintf_s(buffer, sizeof(buffer), "%f", value);
    zulon_print(buffer);
#else
    char buffer[128];
    snprintf(buffer, sizeof(buffer), "%f", value);
    zulon_print(buffer);
#endif
}

// Print a floating-point number with newline
void zulon_println_f64(double value) {
    zulon_print_f64(value);
    zulon_putchar('\n');
}

// ============================================================================
// System Functions
// ============================================================================

// Exit the program with a code
void zulon_exit(int code) {
    exit(code);
}

// Get a character from standard input (simplified)
int zulon_getchar() {
#ifdef _WIN32
    return _getch();
#else
    return getchar();
#endif
}

// Read a line from standard input
// Returns: number of characters read (excluding null terminator)
//         or -1 on error/end of file
// Note: Caller must provide a buffer large enough
//       Maximum line length is maxlen-1 (space for null terminator)
int zulon_read_line(char* buffer, int maxlen) {
    if (buffer == NULL || maxlen <= 0) {
        return -1;
    }

#ifdef _WIN32
    // Windows implementation
    HANDLE hStdIn = GetStdHandle(STD_INPUT_HANDLE);
    if (hStdIn == INVALID_HANDLE_VALUE) {
        return -1;
    }

    int count = 0;
    while (count < maxlen - 1) {
        DWORD read;
        char ch;
        if (!ReadFile(hStdIn, &ch, 1, &read, NULL) || read == 0) {
            break;  // EOF or error
        }

        if (ch == '\n') {
            break;  // End of line
        }

        if (ch == '\r') {
            continue;  // Skip carriage return on Windows
        }

        buffer[count++] = ch;
    }

    buffer[count] = '\0';  // Null terminate
    return count;

#else
    // Unix implementation using fgets
    if (fgets(buffer, maxlen, stdin) == NULL) {
        return -1;  // EOF or error
    }

    // Remove trailing newline if present
    int len = strlen(buffer);
    while (len > 0 && (buffer[len-1] == '\n' || buffer[len-1] == '\r')) {
        buffer[--len] = '\0';
    }

    return len;
#endif
}

// ============================================================================
// String Utility Functions
// ============================================================================

// Get the length of a null-terminated string
// Returns: number of characters (excluding null terminator)
size_t zulon_strlen(const char* str) {
    if (str == NULL) {
        return 0;
    }
    return strlen(str);
}

// Compare two null-terminated strings
// Returns: 0 if strings are equal
//          < 0 if str1 < str2 (lexicographically)
//          > 0 if str1 > str2 (lexicographically)
int zulon_strcmp(const char* str1, const char* str2) {
    if (str1 == NULL && str2 == NULL) {
        return 0;
    }
    if (str1 == NULL) {
        return -1;
    }
    if (str2 == NULL) {
        return 1;
    }
    return strcmp(str1, str2);
}

// ============================================================================
// Reference Counting (for Arc<T>)
// ============================================================================

// Arc<T> memory layout:
// - First field: reference count (atomic int)
// - Remaining fields: the actual data

// Increment reference count
// Note: ptr points to the data, NOT the ref count
// The ref count is located immediately before the data
void zulon_ref_inc(void* ptr) {
    if (ptr == NULL) {
        return;
    }

    // Ref count is stored before the data
    // Cast to int* and decrement to get to the ref count
    int* ref_count = ((int*)ptr) - 1;

    // Increment (not thread-safe for MVP)
    (*ref_count)++;
}

// Decrement reference count and free if zero
// Note: ptr points to the data, NOT the ref count
void zulon_ref_dec(void* ptr) {
    if (ptr == NULL) {
        return;
    }

    // Ref count is stored before the data
    int* ref_count = ((int*)ptr) - 1;

    // Decrement
    (*ref_count)--;

    // Free if this was the last reference
    if (*ref_count <= 0) {
        // Free the entire block (including ref count)
        free(ref_count);
    }
}

// Allocate memory for Arc<T>
// Returns: pointer to the data (ref count is before it)
// Note: Caller must initialize ref count to 1
void* zulon_arc_alloc(size_t data_size) {
    // Allocate extra space for ref count
    size_t total_size = sizeof(int) + data_size;
    void* memory = malloc(total_size);

    if (memory == NULL) {
        return NULL;
    }

    // Initialize ref count to 1
    int* ref_count = (int*)memory;
    *ref_count = 1;

    // Return pointer to data (after ref count)
    return ((char*)memory) + sizeof(int);
}

// ============================================================================
// Memory Allocation Functions
// ============================================================================

// Allocate memory on the heap
void* zulon_runtime_alloc(size_t size) {
    return malloc(size);
}

// Free memory allocated on the heap
void zulon_runtime_free(void* ptr) {
    if (ptr != NULL) {
        free(ptr);
    }
}
