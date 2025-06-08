# Task 1.2: Type System & Conversions

**Status:** 🔴 Not Started  
**Estimated Time:** 4 days  
**Priority:** Critical  
**Assignee:** TBD  

## Description
Implement robust type conversions between Rust types and JavaScript/WASM boundary, ensuring type safety and optimal performance.

## Acceptance Criteria
- [ ] All ladder-rs types have WASM representations
- [ ] Bidirectional conversion functions implemented
- [ ] Error handling for invalid conversions
- [ ] TypeScript definitions match Rust implementations
- [ ] Zero-copy optimizations where possible

## Subtasks

### 1.2.1: Core Type Definitions
**Time Estimate:** 8 hours  
**Status:** 🔴 Not Started

#### Description
Define WASM-compatible versions of core ladder-rs types with proper serialization.

#### Tasks
- [ ] Create `WasmRating` struct with serde support
- [ ] Implement `WasmPlayer` with ID and rating fields
- [ ] Define `WasmTeam` for multi-player scenarios
- [ ] Create `WasmMatchResult` for game outcomes
- [ ] Add `WasmSystemConfig` for rating system parameters

#### Example Implementation
```rust
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WasmRating {
    mean: f64,
    variance: f64,
    system_type: String,
}

#[wasm_bindgen]
impl WasmRating {
    #[wasm_bindgen(constructor)]
    pub fn new(mean: f64, variance: f64, system_type: String) -> WasmRating {
        WasmRating { mean, variance, system_type }
    }

    #[wasm_bindgen(getter)]
    pub fn mean(&self) -> f64 { self.mean }
    
    #[wasm_bindgen(getter)]
    pub fn variance(&self) -> f64 { self.variance }
    
    #[wasm_bindgen(getter)]
    pub fn standard_deviation(&self) -> f64 { self.variance.sqrt() }
    
    #[wasm_bindgen(getter)]
    pub fn conservative_rating(&self) -> f64 { 
        self.mean - 3.0 * self.standard_deviation() 
    }
}
```

---

### 1.2.2: Conversion Implementations
**Time Estimate:** 10 hours  
**Status:** 🔴 Not Started

#### Description
Implement bidirectional conversion functions between native ladder-rs types and WASM types.

#### Tasks
- [ ] Implement `From<EloRating>` for `WasmRating`
- [ ] Implement `From<GlickoRating>` for `WasmRating`
- [ ] Implement `From<TrueSkillRating>` for `WasmRating`
- [ ] Create conversion utilities for collections
- [ ] Add error handling for invalid conversions

#### Conversion Patterns
```rust
impl From<ladder_rs::elo::EloRating> for WasmRating {
    fn from(rating: ladder_rs::elo::EloRating) -> Self {
        WasmRating {
            mean: rating.mean(),
            variance: rating.variance(),
            system_type: "elo".to_string(),
        }
    }
}

impl TryFrom<WasmRating> for ladder_rs::elo::EloRating {
    type Error = JsValue;
    
    fn try_from(wasm_rating: WasmRating) -> Result<Self, Self::Error> {
        if wasm_rating.system_type != "elo" {
            return Err(JsValue::from_str("Invalid system type for Elo rating"));
        }
        Ok(ladder_rs::elo::EloRating::new(wasm_rating.mean))
    }
}
```

---

### 1.2.3: JavaScript Interface Types
**Time Estimate:** 6 hours  
**Status:** 🔴 Not Started

#### Description
Create comprehensive TypeScript definitions that match the WASM interface.

#### Tasks
- [ ] Generate TypeScript interfaces for all WASM types
- [ ] Create utility types for common patterns
- [ ] Add JSDoc documentation with examples
- [ ] Validate TypeScript compilation

#### TypeScript Definitions
```typescript
export interface Rating {
  readonly mean: number;
  readonly variance: number;
  readonly standardDeviation: number;
  readonly conservativeRating: number;
  readonly systemType: 'elo' | 'glicko' | 'trueskill';
}

export interface Player {
  readonly id: string;
  readonly name: string;
  readonly rating: Rating;
  readonly matchHistory: MatchResult[];
}

export interface Team {
  readonly players: Player[];
  readonly averageRating: number;
}

export interface MatchResult {
  readonly teams: Team[];
  readonly outcome: MatchOutcome;
  readonly timestamp: Date;
  readonly matchQuality: number;
}

export type MatchOutcome = 
  | { type: 'win'; winner: number }
  | { type: 'draw' }
  | { type: 'ranked'; ranks: number[] };

export interface SystemConfig {
  readonly systemType: 'elo' | 'glicko' | 'trueskill';
  readonly parameters: Record<string, number>;
}
```

---

### 1.2.4: Serialization Optimization
**Time Estimate:** 6 hours  
**Status:** 🔴 Not Started

#### Description
Optimize serialization/deserialization for performance and minimize data transfer.

#### Tasks
- [ ] Implement custom serde serializers for large objects
- [ ] Use binary formats where appropriate
- [ ] Add compression for bulk data transfers
- [ ] Profile serialization performance

#### Optimization Strategies
```rust
// Use binary serialization for large datasets
use rmp_serde; // MessagePack for binary serialization

#[wasm_bindgen]
pub fn serialize_players_binary(players: &[WasmPlayer]) -> Result<Vec<u8>, JsValue> {
    rmp_serde::to_vec(players)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn deserialize_players_binary(data: &[u8]) -> Result<Vec<WasmPlayer>, JsValue> {
    rmp_serde::from_slice(data)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
```

---

### 1.2.5: Error Handling Framework
**Time Estimate:** 4 hours  
**Status:** 🔴 Not Started

#### Description
Create consistent error handling across the WASM boundary with proper JavaScript error types.

#### Tasks
- [ ] Define WASM-specific error types
- [ ] Implement error conversion utilities
- [ ] Add error context and debugging information
- [ ] Create JavaScript error classes

#### Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WasmError {
    #[error("Invalid rating system: {system}")]
    InvalidSystem { system: String },
    
    #[error("Conversion error: {message}")]
    ConversionError { message: String },
    
    #[error("Rating calculation failed: {source}")]
    CalculationError { 
        #[from] 
        source: ladder_rs::error::Error 
    },
}

impl From<WasmError> for JsValue {
    fn from(error: WasmError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}
```

## Dependencies
- Task 1.1 (WASM Build Configuration) must be completed
- Serde serialization framework
- Understanding of all ladder-rs rating system APIs

## Deliverables
- [ ] Complete type conversion layer in `wasm/src/types.rs`
- [ ] TypeScript definition files
- [ ] Error handling framework
- [ ] Performance benchmarks for serialization
- [ ] Documentation with usage examples

## Risk Factors
- **Medium Risk:** Performance overhead from serialization
- **Low Risk:** Type safety at WASM boundary
- **Low Risk:** Complex nested type conversions

## Testing Checklist
- [ ] All conversion functions work bidirectionally
- [ ] TypeScript definitions compile without errors
- [ ] Error handling provides clear messages
- [ ] Serialization performance meets targets
- [ ] Memory usage is reasonable for large datasets