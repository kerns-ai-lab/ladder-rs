# Task 1.4: WASM Testing Framework

**Status:** 🔴 Not Started  
**Estimated Time:** 3 days  
**Priority:** High  
**Assignee:** TBD  

## Description
Establish comprehensive testing infrastructure for WASM modules, including unit tests, integration tests, and browser environment testing.

## Acceptance Criteria
- [ ] 100% test coverage for WASM bindings
- [ ] Automated testing in browser environments
- [ ] Performance regression testing
- [ ] Cross-browser compatibility verification
- [ ] Integration with CI pipeline

## Subtasks

### 1.4.1: Unit Test Infrastructure
**Time Estimate:** 8 hours  
**Status:** 🔴 Not Started

#### Description
Set up unit testing framework for WASM-specific code using `wasm-bindgen-test`.

#### Tasks
- [ ] Configure `wasm-bindgen-test` in Cargo.toml
- [ ] Create test module structure
- [ ] Implement test utilities and fixtures
- [ ] Add test data generation helpers

#### Test Configuration
```toml
# wasm/Cargo.toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
web-sys = { version = "0.3", features = ["console"] }

[lib]
crate-type = ["cdylib"]

[[test]]
name = "wasm_tests"
path = "tests/wasm_tests.rs"
```

#### Test Structure
```rust
// wasm/tests/wasm_tests.rs
use wasm_bindgen_test::*;
use ladder_rs_wasm::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_rating_creation() {
    let system = WasmRatingSystem::new("elo", None).unwrap();
    let player = system.create_player("test_player");
    
    assert_eq!(player.id(), "test_player");
    assert!(player.rating().mean() > 0.0);
}

#[wasm_bindgen_test]
fn test_rating_updates() {
    let system = WasmRatingSystem::new("trueskill", None).unwrap();
    
    let player1 = system.create_player("alice");
    let player2 = system.create_player("bob");
    
    let team1 = WasmTeam::new(vec![player1]);
    let team2 = WasmTeam::new(vec![player2]);
    
    let outcome = WasmOutcome::win(0);
    let updated = system.update_ratings(vec![team1, team2], outcome).unwrap();
    
    assert!(updated[0].average_rating() > updated[1].average_rating());
}
```

---

### 1.4.2: Browser Integration Tests
**Time Estimate:** 10 hours  
**Status:** 🔴 Not Started

#### Description
Create integration tests that run in actual browser environments to validate WASM behavior.

#### Tasks
- [ ] Set up headless browser testing with Playwright/Puppeteer
- [ ] Create test HTML pages for different scenarios
- [ ] Implement JavaScript test harness
- [ ] Add performance monitoring during tests

#### Browser Test Setup
```javascript
// tests/browser/rating_system_test.js
import init, { WasmRatingSystem } from '../../wasm/pkg/ladder_rs_wasm.js';

describe('Rating System Integration', () => {
    beforeAll(async () => {
        await init();
    });

    test('creates rating system successfully', () => {
        const system = new WasmRatingSystem('elo');
        expect(system).toBeDefined();
    });

    test('handles large player datasets', async () => {
        const system = new WasmRatingSystem('trueskill');
        const players = [];
        
        // Create 1000 players
        for (let i = 0; i < 1000; i++) {
            players.push(system.create_player(`player_${i}`));
        }
        
        expect(players).toHaveLength(1000);
        
        // Measure performance
        const start = performance.now();
        const team1 = players.slice(0, 500);
        const team2 = players.slice(500);
        
        const quality = system.calculate_match_quality([team1, team2]);
        const end = performance.now();
        
        expect(quality).toBeGreaterThan(0);
        expect(end - start).toBeLessThan(100); // < 100ms
    });
});
```

---

### 1.4.3: Cross-Browser Compatibility Testing
**Time Estimate:** 6 hours  
**Status:** 🔴 Not Started

#### Description
Ensure WASM modules work correctly across different browsers and environments.

#### Tasks
- [ ] Set up testing matrix for major browsers
- [ ] Create compatibility test suite
- [ ] Test WebAssembly feature detection
- [ ] Validate performance across browsers

#### Browser Matrix
```yaml
# .github/workflows/browser-tests.yml
strategy:
  matrix:
    browser: [chrome, firefox, safari, edge]
    os: [ubuntu-latest, windows-latest, macos-latest]
    
steps:
  - name: Run browser tests
    run: |
      npm run test:browser -- --browser ${{ matrix.browser }}
```

#### Compatibility Tests
```javascript
// tests/compatibility/feature_detection.js
export function detectWasmSupport() {
    if (typeof WebAssembly === 'object' && 
        typeof WebAssembly.instantiate === 'function') {
        return true;
    }
    return false;
}

export function testBasicOperations() {
    if (!detectWasmSupport()) {
        throw new Error('WebAssembly not supported');
    }
    
    // Test basic WASM operations
    const system = new WasmRatingSystem('elo');
    const player = system.create_player('test');
    
    return {
        creation: !!player,
        rating_access: typeof player.rating().mean() === 'number',
        calculations: true // Add calculation tests
    };
}
```

---

### 1.4.4: Performance Regression Testing
**Time Estimate:** 8 hours  
**Status:** 🔴 Not Started

#### Description
Implement automated performance testing to catch regressions in WASM module performance.

#### Tasks
- [ ] Create performance benchmark suite
- [ ] Set up automated performance monitoring
- [ ] Define performance regression thresholds
- [ ] Integrate with CI pipeline for alerts

#### Performance Benchmarks
```rust
// wasm/benches/wasm_benchmarks.rs
use wasm_bindgen_test::*;
use web_sys::console;

#[wasm_bindgen_test]
fn bench_rating_updates() {
    let system = WasmRatingSystem::new("trueskill", None).unwrap();
    
    let players: Vec<_> = (0..100).map(|i| {
        system.create_player(&format!("player_{}", i))
    }).collect();
    
    let teams = vec![
        WasmTeam::new(players[0..50].to_vec()),
        WasmTeam::new(players[50..].to_vec()),
    ];
    
    let start = js_sys::Date::now();
    
    for _ in 0..100 {
        let outcome = WasmOutcome::win(0);
        let _updated = system.update_ratings(teams.clone(), outcome).unwrap();
    }
    
    let duration = js_sys::Date::now() - start;
    console::log_1(&format!("100 rating updates took {}ms", duration).into());
    
    assert!(duration < 1000.0); // Should complete in under 1 second
}
```

#### CI Performance Monitoring
```yaml
# .github/workflows/performance.yml
name: Performance Tests

on: [push, pull_request]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run performance tests
        run: |
          cd wasm
          wasm-pack test --headless --chrome
          
      - name: Check performance regression
        run: |
          node scripts/check-performance-regression.js
```

## Dependencies
- Task 1.1 (WASM Build Configuration) must be completed
- Task 1.3 (Core API Bindings) must be completed
- Browser testing infrastructure setup

## Deliverables
- [ ] Complete test suite in `wasm/tests/`
- [ ] Browser integration test framework
- [ ] Performance benchmark suite
- [ ] CI pipeline integration
- [ ] Test coverage reporting

## Risk Factors
- **Medium Risk:** Browser compatibility issues
- **Low Risk:** Performance test stability
- **Low Risk:** Test environment setup complexity

## Testing Checklist
- [ ] All unit tests pass in Node.js environment
- [ ] All tests pass in browser environments
- [ ] Performance benchmarks run successfully
- [ ] Test coverage meets 100% target
- [ ] CI pipeline executes tests automatically
- [ ] Cross-browser tests validate compatibility