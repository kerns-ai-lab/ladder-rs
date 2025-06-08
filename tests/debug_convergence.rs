use ladder_rs::trueskill::{
    FactorGraph, GaussianDistribution, GaussianPriorFactor, GaussianComparisonFactor,
};

#[test]
fn debug_schedule_with_comparison_factor() {
    let mut fg = FactorGraph::new();
    let greater_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));
    let lesser_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));

    // Create factors
    let prior_greater = GaussianPriorFactor::new(greater_id, 5.0, 1.0).unwrap();
    let prior_lesser = GaussianPriorFactor::new(lesser_id, 3.0, 1.0).unwrap();
    let comparison = GaussianComparisonFactor::new(greater_id, lesser_id, 0.0, false).unwrap();

    fg.add_factor(Box::new(prior_greater));
    fg.add_factor(Box::new(prior_lesser));
    fg.add_factor(Box::new(comparison));

    println!("Before convergence:");
    let greater_var = fg.get_variable(greater_id).unwrap();
    let lesser_var = fg.get_variable(lesser_id).unwrap();
    println!("Greater variable: mean={}, variance={}", greater_var.value().mean(), greater_var.value().variance());
    println!("Lesser variable: mean={}, variance={}", lesser_var.value().mean(), lesser_var.value().variance());

    let result = fg.run_schedule_loop(1e-6, 5);
    println!("Convergence result: {:?}", result);

    println!("After convergence:");
    let greater_var = fg.get_variable(greater_id).unwrap();
    let lesser_var = fg.get_variable(lesser_id).unwrap();
    println!("Greater variable: mean={}, variance={}", greater_var.value().mean(), greater_var.value().variance());
    println!("Lesser variable: mean={}, variance={}", lesser_var.value().mean(), lesser_var.value().variance());

    println!("Expected greater mean: 5.0, actual: {}", greater_var.value().mean());
    println!("Difference from expected: {}", (greater_var.value().mean() - 5.0).abs());
    println!("Expected lesser mean: 3.0, actual: {}", lesser_var.value().mean());
    println!("Difference from expected: {}", (lesser_var.value().mean() - 3.0).abs());
}