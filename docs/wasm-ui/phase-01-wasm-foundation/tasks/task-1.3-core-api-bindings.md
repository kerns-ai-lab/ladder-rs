# Task 1.3: Core API Bindings

**Status:** 🔴 Not Started  
**Estimated Time:** 3 days  
**Priority:** Critical  
**Assignee:** TBD  

## Description
Implement JavaScript-accessible API bindings for all three rating systems (Elo, Glicko, TrueSkill) with consistent interfaces and error handling.

## Acceptance Criteria
- [ ] All rating systems accessible via unified WASM API
- [ ] Consistent method signatures across systems
- [ ] Proper error propagation to JavaScript
- [ ] Performance optimized for frequent calls
- [ ] Complete feature parity with native Rust API

## Subtasks

### 1.3.1: Unified Rating System Interface
**Time Estimate:** 8 hours  
**Status:** 🔴 Not Started

#### Description
Create a unified WASM interface that can handle all three rating systems through a single API.

#### Tasks
- [ ] Design `WasmRatingSystem` enum for system selection
- [ ] Implement factory methods for each system type
- [ ] Create unified method signatures
- [ ] Add system-specific parameter configuration

#### Implementation Example
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmRatingSystem {
    inner: Box<dyn SystemImpl>,
}

trait SystemImpl {
    fn create_rating(&self) -> WasmRating;
    fn rate(&self, teams: Vec<WasmTeam>, outcome: WasmOutcome) -> Result<Vec<WasmTeam>, JsValue>;
    fn match_quality(&self, teams: Vec<WasmTeam>) -> Result<f64, JsValue>;
    fn system_type(&self) -> String;
}

#[wasm_bindgen]
impl WasmRatingSystem {
    #[wasm_bindgen(constructor)]
    pub fn new(system_type: &str, config: Option<WasmSystemConfig>) -> Result<WasmRatingSystem, JsValue> {
        let inner: Box<dyn SystemImpl> = match system_type {
            "elo" => Box::new(EloSystemImpl::new(config)?),
            "glicko" => Box::new(GlickoSystemImpl::new(config)?),
            "trueskill" => Box::new(TrueSkillSystemImpl::new(config)?),
            _ => return Err(JsValue::from_str("Invalid system type")),
        };
        
        Ok(WasmRatingSystem { inner })
    }

    pub fn create_player(&self, id: &str) -> WasmPlayer {
        WasmPlayer::new(id, self.inner.create_rating())
    }

    pub fn update_ratings(&self, teams: Vec<WasmTeam>, outcome: WasmOutcome) -> Result<Vec<WasmTeam>, JsValue> {
        self.inner.rate(teams, outcome)
    }

    pub fn calculate_match_quality(&self, teams: Vec<WasmTeam>) -> Result<f64, JsValue> {
        self.inner.match_quality(teams)
    }
}
```

---

### 1.3.2: Elo System Implementation
**Time Estimate:** 6 hours  
**Status:** 🔴 Not Started

#### Description
Implement WASM bindings specifically for the Elo rating system.

#### Tasks
- [ ] Create `EloSystemImpl` struct
- [ ] Implement K-factor configuration
- [ ] Add 1v1 match processing
- [ ] Handle Elo-specific parameters

#### Elo-Specific Features
```rust
struct EloSystemImpl {
    system: ladder_rs::elo::EloSystem,
    k_factor: f64,
}

impl EloSystemImpl {
    fn new(config: Option<WasmSystemConfig>) -> Result<Self, JsValue> {
        let k_factor = config
            .and_then(|c| c.get_parameter("k_factor"))
            .unwrap_or(30.0);
            
        Ok(EloSystemImpl {
            system: ladder_rs::elo::EloSystem::new_with_k_factor(k_factor),
            k_factor,
        })
    }
}

impl SystemImpl for EloSystemImpl {
    fn create_rating(&self) -> WasmRating {
        self.system.create_rating().into()
    }
    
    fn rate(&self, teams: Vec<WasmTeam>, outcome: WasmOutcome) -> Result<Vec<WasmTeam>, JsValue> {
        // Convert to native types, process, convert back
        let native_teams: Vec<_> = teams.into_iter()
            .map(|t| t.try_into())
            .collect::<Result<Vec<_>, _>>()?;
            
        let native_outcome = outcome.try_into()?;
        
        let updated = self.system.rate(&native_teams, &native_outcome)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        Ok(updated.into_iter().map(|t| t.into()).collect())
    }
}
```

---

### 1.3.3: Glicko System Implementation  
**Time Estimate:** 6 hours  
**Status:** 🔴 Not Started

#### Description
Implement WASM bindings for both Glicko and Glicko-2 rating systems.

#### Tasks
- [ ] Create `GlickoSystemImpl` struct
- [ ] Support both Glicko and Glicko-2 variants
- [ ] Implement rating period handling
- [ ] Add volatility configuration for Glicko-2

#### Glicko-Specific Features
```rust
struct GlickoSystemImpl {
    system: ladder_rs::glicko::GlickoSystem,
    variant: GlickoVariant,
}

enum GlickoVariant {
    Glicko,
    Glicko2 { tau: f64 },
}

impl GlickoSystemImpl {
    fn new(config: Option<WasmSystemConfig>) -> Result<Self, JsValue> {
        let variant = match config.and_then(|c| c.get_parameter("variant")) {
            Some(2.0) => GlickoVariant::Glicko2 { 
                tau: config.and_then(|c| c.get_parameter("tau")).unwrap_or(0.2) 
            },
            _ => GlickoVariant::Glicko,
        };
        
        let system = match variant {
            GlickoVariant::Glicko => ladder_rs::glicko::GlickoSystem::new(),
            GlickoVariant::Glicko2 { tau } => ladder_rs::glicko::GlickoSystem::new_glicko2(tau),
        };
        
        Ok(GlickoSystemImpl { system, variant })
    }
}
```

---

### 1.3.4: TrueSkill System Implementation
**Time Estimate:** 8 hours  
**Status:** 🔴 Not Started

#### Description
Implement WASM bindings for the TrueSkill rating system with full team support.

#### Tasks
- [ ] Create `TrueSkillSystemImpl` struct
- [ ] Support multi-player teams
- [ ] Implement draw probability configuration
- [ ] Add performance variance tuning

#### TrueSkill-Specific Features
```rust
struct TrueSkillSystemImpl {
    system: ladder_rs::trueskill::TrueSkill,
    beta: f64,
    draw_probability: f64,
}

impl TrueSkillSystemImpl {
    fn new(config: Option<WasmSystemConfig>) -> Result<Self, JsValue> {
        let beta = config
            .and_then(|c| c.get_parameter("beta"))
            .unwrap_or(25.0 / 6.0);
            
        let draw_probability = config
            .and_then(|c| c.get_parameter("draw_probability"))
            .unwrap_or(0.1);
            
        Ok(TrueSkillSystemImpl {
            system: ladder_rs::trueskill::TrueSkill::new_with_params(beta, draw_probability),
            beta,
            draw_probability,
        })
    }
}

impl SystemImpl for TrueSkillSystemImpl {
    fn rate(&self, teams: Vec<WasmTeam>, outcome: WasmOutcome) -> Result<Vec<WasmTeam>, JsValue> {
        // Handle multi-player teams
        let native_teams: Vec<_> = teams.into_iter()
            .map(|team| {
                let players: Result<Vec<_>, _> = team.players.into_iter()
                    .map(|p| p.rating.try_into())
                    .collect();
                players.map(|ps| ladder_rs::trueskill::TrueSkillTeam::from_player_ratings(ps))
            })
            .collect::<Result<Vec<_>, _>>()?;
            
        // Process match...
    }
}
```

## Dependencies
- Task 1.2 (Type System & Conversions) must be completed
- Complete understanding of each rating system's API
- Performance requirements for JavaScript integration

## Deliverables
- [ ] `wasm/src/api/` directory with system implementations
- [ ] Unified JavaScript API interface
- [ ] System-specific configuration options
- [ ] Error handling and validation
- [ ] Performance benchmarks

## Risk Factors
- **Medium Risk:** API complexity across different systems
- **Low Risk:** Performance overhead from abstraction layer
- **Low Risk:** Configuration parameter validation

## Testing Checklist
- [ ] All three systems create and update ratings correctly
- [ ] Match quality calculations work for each system
- [ ] Error handling provides clear feedback
- [ ] Configuration parameters are properly validated
- [ ] Performance meets targets for typical use cases