# TEECore

⚠️ This is just proof of concept and not meant to be deployed. ⚠️

This project is a proof of concept of a software based TEE that runs in core
local caches and uses performance monitoring counters to protect itself against
tampering attempts.

SPOILER: This approach has it's pitfalls! Be aware!

TEECore is a fork of PhipsBoot, which is a relocatable x86_64 bootloader
written in Rust and assembly that loads a kernel into 64-bit mode. It abstracts
a lot of boot-related x86_64 complexity away. It is Multiboot 2 compatible.

TEECore only supports output through serial connection.

## Hardware Requirements
You need a recent Intel CPU to run this. I tested it on Skylake (i5 6600k) and
Raptor Cove (i7 13700k, only P-Cores). As the PMU might differ between serval
micro archs, there is no guarantee that your CPU works.

## Important
TEECore cannot run standalone. It requires some special memory structures that
need to be prepared by other software. This code can be found in the repository
of my thesis.

If you want to try it out, checkout: [My Thesis Repository](https://github.com/scholzp/thesis.git)

## Building
```
make

```

## Other branches
Check out the following branches if you are interested in benchmarking some of
TEECores characteristics. The names of the branches are somewhat misleading.

* bench_mem_constraints
    * Shows how many times the whole memory region must be accessed before all
      of it is cached.
* code_size
    * Installs PMC to monitor misses in the L1I before the environment's code is
      run.
* mem_eval_sc2
    * Can be used to test different heap sizes. L2 Misses will occur if the heap
      spills in other parts of the memory subsystem.
