# ladder-rs-wasm

WebAssembly bindings for the ladder-rs matchmaking library.

## Building

### Prerequisites

- Rust toolchain with wasm32-unknown-unknown target
- wasm-pack (`curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`)
- Node.js (for npm scripts)

### Development Build

```bash
npm run build:dev
# or
./build.sh --dev
```

### Production Build

```bash
npm run build:release
# or
./build.sh --release
```

### Build Options

The build script supports various options:

```bash
./build.sh --help
```

## Build Profiles

- **dev**: Fast builds with debug info (132KB)
- **release**: Optimized for size (21KB)
- **profiling**: Balanced optimization with debug symbols

## Package Structure

```
pkg/
├── ladder_rs_wasm.d.ts      # TypeScript definitions
├── ladder_rs_wasm.js        # JavaScript glue code
├── ladder_rs_wasm_bg.wasm   # WebAssembly binary
└── package.json             # NPM package metadata
```

## Usage

The package can be imported in web applications:

```javascript
import init, { greet } from './pkg/ladder_rs_wasm.js';

async function run() {
    await init();
    greet("WebAssembly");
}

run();
```

## License

MIT