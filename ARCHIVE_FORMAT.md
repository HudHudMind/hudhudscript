## Archive formats

| Feature | Value | Description |
| :--- | :--- | :--- |
| **Software Name** | hudhudscript | The script / software under development |
| **Target Platform** | windows/linux/mac | Compiled for Windows/Linux/Mac operating systems |
| **CPU Architecture** | x86-64 | 64-bit instruction set architecture |
| **CPU Architecture Version** | empty or v3/v4 | Microarchitecture level requiring AVX512 (2017+) |
| **Version** | v0.6.1 | Current Alpha release version |
| **File Extension** | tar.gz/tar.bz2/7z/.zip | Compressed archive format |

### Naming
| Full File Name | Part 1 (Name) | Part 2 (OS) | Part 3 (Arch) | Part 4 (Bits) | Part 5 (Level) | Part 6 HudHudScript Version| part 7 File Extension) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |:---|
| `hudhudscript-linux-x86-64-v4-v0.6.11.tar.gz` | hudhudscript | linux | x86 | 64 |  | v0.6.11|.tar.gz |
| `hudhudscript-windows-x86-64-v4-v0.6.11.zip` | hudhudscript | windows | x86 | 64 | v4 | v0.6.11 |.zip |
| `hudhudscript-linux-x86-64-v4-v0.6.11.tar.gz` | hudhudscript | linux | x86 | 64 | v4 | v0.6.11|.tar.gz |

### Cpu Naming

| Archive | Target | Which Cpu |
| :--- | :--- | :--- |
| x86_64 | Baseline | Any x86-64 |
| x86_64-v3 | AVX2 + SSE4 | 2013+ (Haswell, Zen 1+) |
| x86_64-v4 | AVX512 | 2017+ (Skylake-X, Zen 4+) |
