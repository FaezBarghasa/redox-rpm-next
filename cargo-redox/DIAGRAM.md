# Architecture Diagrams - cargo-redox

## High-Level Execution Flow

```mermaid
flowchart TD
    A[Invocation: cargo-redox / cargo redox] --> B[Parse Arguments & Options]
    B --> C{Profile Specified?}
    C -- CLI Flag --redox-profile --> D[Extract Profile Name]
    C -- REDOX_PROFILE Env --> D
    C -- None --> E[Default Profile: 'pro']
    D --> F[Map Profile to Target Triple]
    E --> F
    F --> G{Target == xtensa-esp32-none-elf?}
    G -- Yes --> H[Append -C link-arg=-nostartfiles to RUSTFLAGS]
    G -- No --> I[Keep standard environment]
    H --> J[Inject --target <triple> into Cargo invocation]
    I --> J
    J --> K[Spawn cargo process]
    K --> L[Forward exit code to caller]
```

## Profile Mapping Structure

```mermaid
classDiagram
    class CargoRedox {
        +main()
        +determine_profile(args: Vec~String~) String
        +determine_target(profile: String) String
        +inject_cargo_args(cmd: Command, args: Vec~String~)
    }

    class Profile {
        <<enumeration>>
        NANO
        PRO
        TITAN
    }

    class TargetTriple {
        <<enumeration>>
        XTENSA_ESP32_NONE_ELF
        X86_64_UNKNOWN_REDOX
    }

    CargoRedox ..> Profile
    CargoRedox ..> TargetTriple
```
