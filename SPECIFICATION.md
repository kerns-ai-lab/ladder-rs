# Rust Matchmaking Library Specification

The goal of this project is to develop a high-performance, extensible matchmaking library in Rust. The initial focus will be a comprehensive implementation of the TrueSkill algorithm, followed by support for Elo and Glicko/Glicko-2. The library must be modular, performant, mathematically sound, type-safe, and ergonomic. Extensibility will be achieved primarily through Rust's trait system.

**Core Design Principles:**
* [cite_start]**Modularity**: Each rating system (TrueSkill, Elo, Glicko) will be a self-contained module.
* [cite_start]**Extensibility**: New rating algorithms can be easily integrated via traits.
* [cite_start]**Performance**: Computational processes will be optimized, leveraging Rust's capabilities.
* [cite_start]**Accuracy & Mathematical Soundness**: Implementations will strictly adhere to the algorithms' mathematical models.
* [cite_start]**Type Safety**: Rust's static type system will be fully utilized.
* [cite_start]**Ergonomics**: The public API will be intuitive and user-friendly.

**Constraints:**
* **Language**: Rust.
* **No Code Reuse for Core Algorithms**: Do **not** copy or directly import existing open-source skill rating libraries to fulfill the requirements for implementing the TrueSkill, Elo, or Glicko algorithms themselves. The core logic for these algorithms must be implemented from scratch based on this specification and the underlying mathematical principles described in the provided sources.
* **Allowed Dependencies**: Standard Rust libraries and external crates for mathematical and statistical functions (e.g., for Gaussian distributions, matrix operations if needed) are permitted.
* [cite_start]**Numerical Precision**: Default to `f64` for all core rating calculations to ensure accuracy and mitigate potential `FloatingPointError` issues.
* [cite_start]**Error Handling**: Implement robust error handling using `Result<T, E>` and custom error types.
* **Concurrency**: The library design should not preclude concurrent use. [cite_start]`RatingSystem` instances should be stateless regarding individual matches, allowing parallel execution of rating updates for different matches.

---
## Implementation Phases

### Phase 1: Core Abstractions & Library Foundation

**Objective**: Establish the foundational traits and types for the library.

**Tasks**:
1.  **Define Core Traits**:
    * [cite_start]`RatingSystem`: Trait defining common operations for all rating systems (e.g., initializing ratings, updating ratings, calculating match quality).
        * [cite_start]Use associated types for `PlayerRating`, `TeamRating` (if applicable), and `Outcome`.
    * [cite_start]`Rating`: Trait for individual player skill representation (e.g., methods to access mean, variance, etc.).
    * [cite_start]`TeamRating`: Trait for team-specific rating properties (if needed beyond a collection of `PlayerRating`).
    * [cite_start]`Outcome`: Trait to represent game outcomes (win, loss, draw, ranks).
2.  **Basic Data Structures**:
    * Define common structs for player identifiers, team compositions, and game results.
    * [cite_start]Game outcome (ranks) representation: "Game outcome should be represented as a `Vec<usize>`, where each element is a rank (1st, 2nd, etc.), and the index of the element corresponds to the team's index in the input `rating_groups`." 
3.  **Error Handling Module**:
    * [cite_start]Implement custom error types (`enum Error`) for the library, covering invalid inputs, numerical issues, etc..
4.  **Project Structure**:
    * [cite_start]Set up the Rust project with a clear module structure (e.g., `core`, `trueskill`, `elo`, `glicko`).

---
### Phase 2: TrueSkill Implementation - Foundational Elements

**Objective**: Implement the basic building blocks for the TrueSkill algorithm. This phase focuses on data representation and initial setup.
*Constraint Reminder: The TrueSkill algorithm itself must be implemented from scratch based on this specification and its sources.*

**Tasks**:
1.  **TrueSkill-Specific Rating Structures**:
    * [cite_start]Implement structs representing a player's skill in TrueSkill:
        * [cite_start]Mean ($\mu$) 
        * [cite_start]Variance ($\sigma^2$) 
        * [cite_start]Precision ($\pi$), calculated as $\pi = 1/\sigma^2$.
        * [cite_start]Precision-Adjusted Mean ($\tau_{pam}$), calculated as $\tau_{pam} = \pi\mu$. (Using $\tau_{pam}$ to distinguish from dynamics variance $\gamma^2$).
    * Ensure these structures implement the `Rating` trait.
2.  **Parameter Representation**:
    * Implement structures or types to hold TrueSkill global parameters:
        * [cite_start]Initial $\mu_0$ (default: 25).
        * [cite_start]Initial $\sigma_0^2$ (default: $(25/3)^2$).
        * [cite_start]Performance Variance ($\beta^2$, default: $(\sigma_0/2)^2$).
        * [cite_start]Dynamics Variance (use `gamma_squared`, default: $(\sigma_0/100)^2$)[cite: 93, 181, 295]. [cite_start](This is referred to as $\gamma^2$ in the TrueSkill paper ).
    * [cite_start]Provide functions to initialize new players with default ratings: $\mu = \mu_0$, $\sigma^2 = \sigma_0^2$.
3.  **Factor Graph Primitives (Conceptual)**:
    * Define internal structures for representing:
        * [cite_start]Variable Nodes (player skills $s_i$, performances $p_i$, team performances $t_j$, performance differences $d_k$).
        * [cite_start]Factor Nodes (priors, likelihoods, sums, differences, comparisons).
        * [cite_start]Messages (1D Gaussians, typically represented by mean and variance, or precision and precision-adjusted mean for computational efficiency).
    * [cite_start]These will be internal to the TrueSkill module.

---
### Phase 3: TrueSkill Implementation - Core Algorithm & Message Passing

[cite_start]**Objective**: Implement the core TrueSkill rating update logic using factor graphs and message passing.
*Constraint Reminder: The TrueSkill algorithm itself must be implemented from scratch based on this specification and its sources.*

**Tasks**:
1.  **Message Passing Scheduler**:
    * Implement the logic to manage the iterative message passing schedule. [cite_start]The TrueSkill paper describes this as: First, "light arrow" messages are updated from top to bottom (e.g., skill prior $s_i \rightarrow p_i$, $p_i \rightarrow t_j$). [cite_start]Then, an iterative loop over team performance $t_j$ and performance difference $d_k$ nodes occurs (messages 1-6 in Figure 1 of the TrueSkill paper). [cite_start]This loop is needed due to approximate messages from comparison factors. [cite_start]Finally, "dark arrow" messages update from bottom to top (e.g., $p_i \rightarrow s_i$) to compute skill posteriors. [cite_start]This schedule should be implemented based on Figure 1 of the NIPS-2006 TrueSkill paper.
2.  [cite_start]**Factor Implementations (Message Update Logic)**: (Based on Table 1 from the NIPS-2006 TrueSkill paper and surrounding text )
    * [cite_start]**Gaussian Prior Factors**: Connect to skill variable $s_i$, incorporating prior $\mathcal{N}(s_i; \mu_i, \sigma_i^2)$.
    * [cite_start]**Gaussian Likelihood Factors (Performance)**: Model $p_i \sim \mathcal{N}(p_i; s_i, \beta^2)$.
    * [cite_start]**Gaussian Weighted Sum Factors (Team Performance)**: Model $t_j = \sum_{i \in A_j} p_i$. [cite_start]Support partial play weights if specified  (though the primary TrueSkill paper focuses on $t_j = \sum p_i$).
    * [cite_start]**Gaussian Difference Factors (Performance Difference)**: Model $d_k = t_A - t_B$.
    * [cite_start]**Comparison Factors (Win/Draw Outcome)**: Connect performance differences $d_k$ to outcomes ($\mathbb{I}(d_k > \epsilon_{margin})$ or $\mathbb{I}(|d_k| \le \epsilon_{margin})$)[cite: 28, 31, 50]. [cite_start]Messages are approximated using moment matching (Expectation Propagation)[cite: 51, 203]. [cite_start]The update for the marginal of $d_k$ (denoted as variable $x$ in Table 1 [cite: 54][cite_start]) uses $V_f$ and $W_f$ functions.
        Let $d_{cavity}$ and $c_{cavity}$ be the mean and variance of the cavity distribution for $x$ (the performance difference). The arguments to the $V_f$ and $W_f$ functions are $t_{arg} = d_{cavity} / \sqrt{c_{cavity}}$ and $\epsilon_{arg} = \epsilon_{game} \sqrt{c_{cavity}}$. (Derived from Table 1 context in the NIPS-2006 TrueSkill paper: Table 1 defines arguments $d/\sqrt{c}$ and $\epsilon\sqrt{c}$ for $V_f, W_f$, where $d := \tau_x - \tau_{f \rightarrow x}$ and $c := \pi_x - \pi_{f \rightarrow x}$ relate to parameters of the cavity distribution $p(x)/m_{x \rightarrow f}(x)$).

        The functions are (where $\mathcal{N}(\cdot)$ is the standard Normal PDF, $\Phi(\cdot)$ is the standard Normal CDF, and $\epsilon_{game}$ is the game's draw margin):
        For a **WIN** factor ($\mathbb{I}(x > \epsilon_{game})$):
        $V_{\text{win}}(t, \epsilon_{\text{effective}}) = \frac{\mathcal{N}(t-\epsilon_{\text{effective}})}{\Phi(t-\epsilon_{\text{effective}})}$ 
        $W_{\text{win}}(t, \epsilon_{\text{effective}}) = V_{\text{win}}(t, \epsilon_{\text{effective}}) \cdot (V_{\text{win}}(t, \epsilon_{\text{effective}}) + t-\epsilon_{\text{effective}})$ 

        For a **DRAW** factor ($\mathbb{I}(|x| \le \epsilon_{game})$):
        $V_{\text{draw}}(t, \epsilon_{\text{effective}}) = \frac{\mathcal{N}(-\epsilon_{\text{effective}}-t) - \mathcal{N}(\epsilon_{\text{effective}}-t)}{\Phi(\epsilon_{\text{effective}}-t) - \Phi(-\epsilon_{\text{effective}}-t)}$ 
        $W_{\text{draw}}(t, \epsilon_{\text{effective}}) = V_{\text{draw}}(t, \epsilon_{\text{effective}})^2 + \frac{(\epsilon_{\text{effective}}-t)\mathcal{N}(\epsilon_{\text{effective}}-t) + (\epsilon_{\text{effective}}+t)\mathcal{N}(\epsilon_{\text{effective}}+t)}{\Phi(\epsilon_{\text{effective}}-t) - \Phi(-\epsilon_{\text{effective}}-t)}$ 

        The updated parameters for the marginal of $x$ (e.g., $\pi_x^{new}, \tau_x^{new}$) are then calculated using these $V_f, W_f$ values according to the bottom rule in Table 1 of the NIPS-2006 TrueSkill paper.
3.  **Skill Update Mechanism**:
    * Integrate factor updates and the message passing scheduler for a full skill update cycle.
    * Implement Gaussian density filtering: the posterior skill distribution after one game becomes the prior for the next. An additive dynamics variance $\gamma^2$ can be added to the prior variance if skills vary over time.
4.  **Convergence**:
    * Implement iterative message passing for a single game update. Iterations continue until changes in skill marginals (mean and variance) are below a predefined epsilon (e.g., 0.0001), or a maximum number of iterations (e.g., 10-20) is reached.

---
### Phase 4: TrueSkill Implementation - Features & API

**Objective**: Complete the TrueSkill module by adding remaining features and exposing them via the `RatingSystem` trait.
*Constraint Reminder: The TrueSkill algorithm itself must be implemented from scratch based on this specification and its sources.*

**Tasks**:
1.  **Implement `RatingSystem` for TrueSkill**:
    * [cite_start]`create_rating()`: Returns a new TrueSkill player rating with default values ($\mu_0, \sigma_0^2$).
    * [cite_start]`rate()`: Takes rating groups and game outcome (ranks as defined in Phase 1) and returns updated ratings.
        * [cite_start]Handle team-based matches: Team performance $t_j = \sum_{i \in A_j} p_i$. [cite_start]Infer individual skills from team results.
        * Handle draws explicitly: A draw $r_{(j)}=r_{(j+1)}$ means $|t_{r_{(j)}}-t_{r_{(j+1)}}| [cite_start]\le \epsilon_{game}$.
            * [cite_start]$\epsilon_{game}$ can be set by relating it to an empirical `draw_probability`[cite: 28, 63, 261]. [cite_start]The NIPS-2006 TrueSkill paper provides the formula [cite: 63] [cite_start](also in library spec ):
                `draw_probability` = $\Phi\left(\frac{\epsilon_{game}}{\sigma_{\text{diff}}}\right) - \Phi\left(\frac{-\epsilon_{game}}{\sigma_{\text{diff}}}\right)$
                where $\sigma_{\text{diff}}$ is the standard deviation of the performance difference. The Halo 2 experiments used a denominator related to $n_1, n_2$ (players per team) and $\beta^2$ (performance variance). The earlier library specification document indicated using $\sigma_{\text{diff}} = \sqrt{2\beta^{2}+n_{1}\beta^{2}+n_{2}\beta^{2}}$. This should be used if deriving $\epsilon_{game}$.
    * `calculate_match_quality()`: Implement the pairwise match quality formula (Equation 7 in NIPS-2006 TrueSkill paper , also in library spec ):
        `match_quality` = $\sqrt{\frac{2\beta^{2}}{2\beta^{2}+\sigma_{i}^{2}+\sigma_{j}^{2}}} \cdot \exp\left(-\frac{(\mu_{i}-\mu_{j})^{2}}{2(2\beta^{2}+\sigma_{i}^{2}+\sigma_{j}^{2})}\right)$
        where $\mu_i, \sigma_i^2$ are player skill parameters and $\beta^2$ is performance variance.
2.  **Configuration**:
    * Allow configuration of TrueSkill parameters: $\beta^2$ (performance variance) , `gamma_squared` (dynamics variance) , and `draw_probability` (or $\epsilon_{game}$ directly).
3.  **Leaderboard Display**:
    * Provide a utility function to calculate the conservative skill estimate: $\mu_i - 3\sigma_i$.
4.  **Documentation & Limitations**:
    * Document the TrueSkill implementation, its parameters, and known limitations (e.g., additive team performance assumption, which may fail in certain game modes like Capture-the-Flag in "Small Teams" ).

---
### Phase 5: Elo Rating System Implementation

**Objective**: Implement the Elo rating system as a module conforming to the `RatingSystem` trait.
*Constraint Reminder: The Elo algorithm itself must be implemented from scratch based on this specification and its sources.*

**Tasks**:
1.  **Elo-Specific Rating Structure**:
    * [cite_start]Implement a struct for Elo rating (single numerical value $s$).
    * Ensure it implements the `Rating` trait.
2.  **Implement `RatingSystem` for Elo**:
    * `create_rating()`: Returns a new Elo player rating (e.g., default starting value like 1500).
    * `rate()`:
        * Input: Two players (ratings $s_1, s_2$). Focus on 1v1 matches.
        * [cite_start]Outcome: $y=+1$ if player 1 wins, $y=-1$ if player 2 wins, $y=0$ for a draw.
        * [cite_start]Update ratings: $s_1 \leftarrow s_1 + y\Delta$, $s_2 \leftarrow s_2 - y\Delta$.
        * [cite_start]Magnitude $\Delta$ (from NIPS-2006 TrueSkill paper, Eq. after (1) [cite: 12][cite_start], also in library spec ):
            $\Delta = \frac{\alpha\beta_{elo}\sqrt{\pi}}{\text{K-Factor}}\left(\frac{y+1}{2} - \Phi\left(\frac{s_1 - s_2}{\sqrt{2}\beta_{elo}}\right)\right)$
            where $\Phi$ is Gaussian CDF, $0 < \alpha < 1$ (e.g., 0.05-0.1) , $\beta_{elo}$ is performance variance for Elo context, K-Factor is the scaling constant (e.g., 10-30).
    * `calculate_match_quality()`:
        * Probability of player 1 winning (Equation 1 in NIPS-2006 TrueSkill paper , also in library spec ):
            $P(\text{player 1 wins}) = \Phi\left(\frac{s_1 - s_2}{\sqrt{2}\beta_{elo}}\right)$
            Match quality is higher when this is near 0.5.
3.  **Configuration**:
    * Allow configuration of K-Factor, $\alpha$, and $\beta_{elo}$.
4.  **Documentation**:
    * Document the Elo implementation. Note that most current Elo variants use a logistic distribution , while this specification uses Gaussian as per the provided TrueSkill paper's Elo description.

---
### Phase 6: Glicko & Glicko-2 Rating Systems Implementation

**Objective**: Implement Glicko and Glicko-2 as modules conforming to the `RatingSystem` trait.
*Constraint Reminder: The Glicko/Glicko-2 algorithms must be implemented from scratch.*

**Tasks**:
1.  **Glicko-Specific Rating Structures**:
    * [cite_start]**Glicko**: Mean ($\mu$) and Rating Deviation (RD, analogous to standard deviation $\sigma$, whose square is variance $\sigma^2$).
    * [cite_start]**Glicko-2**: Extends Glicko by adding Rating Volatility ($\sigma_{volatility}$).
    * Ensure these implement the `Rating` trait.
2.  **Implement `RatingSystem` for Glicko/Glicko-2**:
    * `create_rating()`: Returns new ratings with defaults (e.g., Glicko: $\mu=1500, RD=350$; Glicko-2: $\mu=1500, RD=350, \sigma_{volatility}=0.06$).
    * `rate()`:
        * Implement the Glicko/Glicko-2 update algorithms.
        * [cite_start]**Note**: The provided source documents describe Glicko conceptually  [cite_start]and Glicko-2 by its addition of volatility. Full step-by-step mathematical formulas for Glicko or Glicko-2 updates are not detailed. Implementation requires referring to Mark Glickman's original papers (e.g., [5] in NIPS-2006 paper for Glicko) for detailed algorithms. Implement per-match updates.
    * `calculate_match_quality()`:
        * Base on outcome probabilities given player ratings. Refer to Glickman's papers for formulas.
3.  **Configuration**:
    * Allow configuration of system constants (e.g., Glicko-2 system constant $\tau_{g2}$ for volatility change, which corresponds to `tau` in library spec source [332]).
4.  **Documentation**:
    * Document implementations, parameters, and clearly state reliance on Glickman's papers for full algorithm details.

---
### Phase 7: Finalization, Performance, Testing & Documentation

**Objective**: Ensure the library is robust, performant, well-tested, and well-documented.

**Tasks**:
1.  [cite_start]**Performance Optimization**: Review critical paths, leverage Rust's zero-cost abstractions [cite: 368, 369][cite_start], ensure efficient memory use.
2.  [cite_start]**Testing**: Comprehensive unit, integration, and property-based tests. Test edge cases. Validate against known results/reference outputs (e.g., from TrueSkill.org, Moserware blog, Glickman's examples).
3.  [cite_start]**Benchmarking**: For rating updates and match quality calculations.
4.  [cite_start]**`MathBackend` Trait (Optional)**: If pursued, design and implement a `MathBackend` trait for swappable numerical implementations.
5.  [cite_start]**API Refinement & Ergonomics**: Review public API for clarity, consistency, ease of use.
6.  **Comprehensive Documentation**:
    * API Documentation (`rustdoc`).
    * [cite_start]Usage Examples.
    * [cite_start]Parameter Tuning Guide: Explain impact of parameters (TrueSkill's $\beta^2, \text{gamma_squared}$; Elo's K-factor; Glicko-2's $\tau_{g2}$).
    * [cite_start]Document known limitations.

---
## Parallel Development Strategy

* **Phase 1** must be completed first.
* Upon completion of Phase 1:
    * **Team A**: **Phase 2 (TrueSkill Foundations)**, then **Phase 3 (TrueSkill Core Algorithm)**, then **Phase 4 (TrueSkill Features & API)**.
    * **Team B**: **Phase 5 (Elo Implementation)**.
    * **Team C**: **Phase 6 (Glicko/Glicko-2 Implementation)**.
* **Phase 7 (Finalization)** will be an integrated effort.

