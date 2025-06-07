use ladder_rs::trueskill::{GaussianComparisonFactor, Factor};
use statrs::distribution::{Normal, ContinuousCDF, Continuous};

#[test]
fn test_draw_message_construction() {
    let greater_id = 0;
    let lesser_id = 1;
    let draw_margin = 0.5;

    let mut factor = GaussianComparisonFactor::new(greater_id, lesser_id, draw_margin, true).unwrap();
    let _ = factor.update_message(greater_id).unwrap();

    let msg = factor.message_to(greater_id).unwrap().value();

    let normal = Normal::new(0.0, 1.0).unwrap();
    let phi_upper = normal.cdf(draw_margin);
    let phi_lower = normal.cdf(-draw_margin);
    let denom = phi_upper - phi_lower;
    let w_draw = if denom.abs() < 1e-10 {
        0.0
    } else {
        let pdf = normal.pdf(draw_margin);
        2.0 * draw_margin * pdf / denom
    };
    let expected_precision = 1.0 - w_draw;

    assert!((msg.precision_mean() - 0.0).abs() < 1e-10);
    assert!((msg.precision() - expected_precision).abs() < 1e-10);
}
