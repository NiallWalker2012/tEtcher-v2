fn main() {
    cc::Build::new()
        .file("c_src/flash.c")
        .include("c_src")
        .opt_level(3)          // same as -O3
        .flag("-march=native") // use CPU-specific instructions
        .flag("-funroll-loops")
        .flag("-Ofast")        // optional: fastest non-strict math
        .compile("flash");
}
