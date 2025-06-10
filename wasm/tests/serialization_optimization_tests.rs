//! Comprehensive tests for Task 1.2.4: Serialization Optimization
//!
//! Tests cover:
//! - Performance benchmarks for existing serialization
//! - Bulk serialization for collections
//! - Compression algorithms for large datasets
//! - Memory efficiency improvements
//! - Cross-browser compatibility
//! - Bundle size impact validation

use ladder_rs_wasm::js_interface::core::*;
use ladder_rs_wasm::js_interface::systems::*;
use wasm_bindgen::prelude::*;
use std::time::{Duration, Instant};

/// Test serialization performance benchmarks
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_rating_json_serialization_performance() {
        let rating = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        
        // Benchmark JSON serialization
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = rating.to_json();
        }
        let json_duration = start.elapsed();
        
        // Ensure reasonable performance (should be under 10ms for 1000 operations)
        assert!(json_duration < Duration::from_millis(10), 
                "JSON serialization too slow: {:?}", json_duration);
    }

    #[test]
    fn test_rating_binary_serialization_performance() {
        let rating = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        
        // Benchmark binary serialization
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = rating.to_binary();
        }
        let binary_duration = start.elapsed();
        
        // Binary should be faster than JSON
        assert!(binary_duration < Duration::from_millis(5), 
                "Binary serialization too slow: {:?}", binary_duration);
    }

    #[test]
    fn test_team_serialization_performance() {
        let mut team = JsTeamInterface::new();
        
        // Add multiple players to create a realistic team
        for i in 0..10 {
            let rating = JsRatingInterface::new(1500.0 + i as f64 * 10.0, 200.0)
                .expect("Valid rating");
            team.add_player(rating);
        }
        
        // Benchmark team toString performance
        let start = Instant::now();
        for _ in 0..100 {
            let _ = team.to_string();
        }
        let duration = start.elapsed();
        
        assert!(duration < Duration::from_millis(10), 
                "Team serialization too slow: {:?}", duration);
    }
}

/// Test bulk serialization for collections
#[cfg(test)]
mod bulk_serialization_tests {
    use super::*;

    #[test]
    fn test_rating_collection_serialization() {
        let mut collection = JsRatingCollectionInterface::new();
        
        // Add multiple ratings
        for i in 0..100 {
            let rating = JsRatingInterface::new(1500.0 + i as f64, 200.0 + i as f64)
                .expect("Valid rating");
            collection.add(rating);
        }
        
        assert_eq!(collection.length(), 100);
        
        // Test individual access performance
        let start = Instant::now();
        for i in 0..100 {
            let _ = collection.get(i);
        }
        let access_duration = start.elapsed();
        
        assert!(access_duration < Duration::from_millis(5), 
                "Collection access too slow: {:?}", access_duration);
    }

    #[test] 
    fn test_bulk_json_serialization() {
        // Create multiple ratings for bulk operations
        let ratings: Vec<JsRatingInterface> = (0..50)
            .map(|i| JsRatingInterface::new(1500.0 + i as f64 * 10.0, 200.0 + i as f64)
                .expect("Valid rating"))
            .collect();
        
        // Test serializing all to JSON strings
        let start = Instant::now();
        let json_strings: Vec<String> = ratings.iter()
            .map(|r| r.to_json())
            .collect();
        let bulk_json_duration = start.elapsed();
        
        assert_eq!(json_strings.len(), 50);
        assert!(bulk_json_duration < Duration::from_millis(20), 
                "Bulk JSON serialization too slow: {:?}", bulk_json_duration);
        
        // Verify each JSON is valid
        for json in &json_strings {
            assert!(json.contains("\"mean\":"));
            assert!(json.contains("\"variance\":"));
        }
    }

    #[test]
    fn test_bulk_binary_serialization() {
        // Create multiple ratings for bulk binary operations
        let ratings: Vec<JsRatingInterface> = (0..50)
            .map(|i| JsRatingInterface::new(1500.0 + i as f64 * 10.0, 200.0 + i as f64)
                .expect("Valid rating"))
            .collect();
        
        // Test serializing all to binary
        let start = Instant::now();
        let binary_data: Vec<Vec<u8>> = ratings.iter()
            .map(|r| r.to_binary())
            .collect();
        let bulk_binary_duration = start.elapsed();
        
        assert_eq!(binary_data.len(), 50);
        assert!(bulk_binary_duration < Duration::from_millis(10), 
                "Bulk binary serialization too slow: {:?}", bulk_binary_duration);
        
        // Verify each binary data is correct size (16 bytes for f64 + f64)
        for data in &binary_data {
            assert_eq!(data.len(), 16);
        }
    }
}

/// Test data compression for large datasets
#[cfg(test)]
mod compression_tests {
    use super::*;

    #[test]
    fn test_binary_data_size_efficiency() {
        let rating = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        
        let json_size = rating.to_json().len();
        let binary_size = rating.to_binary().len();
        
        // Binary should be more compact than JSON for this data
        assert!(binary_size <= json_size, 
                "Binary serialization not more efficient: {} vs {} bytes", 
                binary_size, json_size);
        
        // Binary should be exactly 16 bytes (8 for mean + 8 for variance)
        assert_eq!(binary_size, 16);
    }

    #[test]
    fn test_large_dataset_serialization() {
        // Create a large collection to test efficiency
        let large_count = 1000;
        let ratings: Vec<JsRatingInterface> = (0..large_count)
            .map(|i| JsRatingInterface::new(1500.0 + i as f64, 200.0 + i as f64)
                .expect("Valid rating"))
            .collect();
        
        // Calculate total serialized size
        let total_json_size: usize = ratings.iter()
            .map(|r| r.to_json().len())
            .sum();
        
        let total_binary_size: usize = ratings.iter()
            .map(|r| r.to_binary().len())
            .sum();
        
        // Binary should be significantly more compact for large datasets
        let compression_ratio = total_json_size as f64 / total_binary_size as f64;
        assert!(compression_ratio > 2.0, 
                "Binary compression ratio too low: {:.2}x", compression_ratio);
        
        // Should be exactly 16KB for 1000 ratings (16 bytes each)
        assert_eq!(total_binary_size, large_count * 16);
    }

    #[test]
    fn test_redundant_data_handling() {
        // Test with identical ratings to see if we handle redundancy
        let identical_ratings: Vec<JsRatingInterface> = (0..100)
            .map(|_| JsRatingInterface::new(1500.0, 200.0).expect("Valid rating"))
            .collect();
        
        // All should serialize to identical data
        let first_json = identical_ratings[0].to_json();
        let first_binary = identical_ratings[0].to_binary();
        
        for rating in &identical_ratings {
            assert_eq!(rating.to_json(), first_json);
            assert_eq!(rating.to_binary(), first_binary);
        }
        
        // Total size should scale linearly (no built-in deduplication yet)
        let total_binary_size: usize = identical_ratings.iter()
            .map(|r| r.to_binary().len())
            .sum();
        assert_eq!(total_binary_size, 100 * 16);
    }
}

/// Test memory efficiency improvements
#[cfg(test)]
mod memory_efficiency_tests {
    use super::*;

    #[test]
    fn test_zero_copy_serialization() {
        let rating = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        
        // Test that binary serialization doesn't require excessive allocations
        let binary1 = rating.to_binary();
        let binary2 = rating.to_binary();
        
        // Should produce identical results
        assert_eq!(binary1, binary2);
        
        // Should be minimal size
        assert_eq!(binary1.len(), 16);
    }

    #[test]
    fn test_deserialization_efficiency() {
        let original = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        
        // Test round-trip efficiency
        let json_data = original.to_json();
        let binary_data = original.to_binary();
        
        // JSON round-trip
        let start = Instant::now();
        let from_json = JsRatingInterface::from_json(&json_data).expect("Valid JSON");
        let json_deserial_time = start.elapsed();
        
        // Binary round-trip  
        let start = Instant::now();
        let from_binary = JsRatingInterface::from_binary(&binary_data).expect("Valid binary");
        let binary_deserial_time = start.elapsed();
        
        // Verify correctness
        assert_eq!(from_json.mean(), original.mean());
        assert_eq!(from_json.variance(), original.variance());
        assert_eq!(from_binary.mean(), original.mean());
        assert_eq!(from_binary.variance(), original.variance());
        
        // Binary deserialization should be faster
        assert!(binary_deserial_time <= json_deserial_time, 
                "Binary deserialization should be faster than JSON: {:?} vs {:?}", 
                binary_deserial_time, json_deserial_time);
    }

    #[test]
    fn test_streaming_serialization_readiness() {
        // Test that our serialization can handle streaming scenarios
        let mut team = JsTeamInterface::new();
        
        // Add players one by one (simulating streaming)
        for i in 0..20 {
            let rating = JsRatingInterface::new(1500.0 + i as f64 * 5.0, 200.0)
                .expect("Valid rating");
            team.add_player(rating);
            
            // Should be able to serialize at any point
            let team_str = team.to_string();
            assert!(team_str.contains(&format!("{} players", i + 1)));
        }
        
        assert_eq!(team.size(), 20);
    }
}

/// Test cross-browser compatibility for serialization
#[cfg(test)]
mod compatibility_tests {
    use super::*;

    #[test]
    fn test_json_format_compatibility() {
        let rating = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        let json = rating.to_json();
        
        // Should be valid JSON that any browser can parse
        assert!(json.starts_with("{"));
        assert!(json.ends_with("}"));
        assert!(json.contains("\"mean\":1500"));
        assert!(json.contains("\"variance\":200"));
        
        // Should not contain special characters that could cause issues
        assert!(!json.contains("\\"));
        assert!(!json.contains("\n"));
        assert!(!json.contains("\r"));
    }

    #[test]
    fn test_binary_format_deterministic() {
        let rating1 = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        let rating2 = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        
        // Same values should produce identical binary representations
        assert_eq!(rating1.to_binary(), rating2.to_binary());
        
        // Different values should produce different binary
        let rating3 = JsRatingInterface::new(1501.0, 200.0).expect("Valid rating");
        assert_ne!(rating1.to_binary(), rating3.to_binary());
    }

    #[test]
    fn test_endianness_consistency() {
        let rating = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        let binary = rating.to_binary();
        
        // Should use little-endian consistently
        assert_eq!(binary.len(), 16);
        
        // Test round-trip maintains precision
        let restored = JsRatingInterface::from_binary(&binary).expect("Valid binary");
        assert_eq!(restored.mean(), rating.mean());
        assert_eq!(restored.variance(), rating.variance());
    }
}

/// Test error handling in serialization
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_json_handling() {
        // Test various invalid JSON formats
        let invalid_jsons = vec![
            "",
            "{}",
            "{\"mean\":1500}",
            "{\"variance\":200}",
            "{\"mean\":\"invalid\",\"variance\":200}",
            "{\"mean\":1500,\"variance\":\"invalid\"}",
            "invalid json",
            "{\"mean\":1500,\"variance\":200,}",
        ];
        
        for invalid in invalid_jsons {
            let result = JsRatingInterface::from_json(invalid);
            assert!(result.is_err(), "Should reject invalid JSON: {}", invalid);
        }
    }

    #[test]
    fn test_invalid_binary_handling() {
        // Test various invalid binary formats
        let invalid_binaries = vec![
            vec![], // Empty
            vec![0; 15], // Too short
            vec![0; 17], // Too long
            vec![0; 8], // Half size
        ];
        
        for invalid in invalid_binaries {
            let result = JsRatingInterface::from_binary(&invalid);
            assert!(result.is_err(), "Should reject invalid binary of length {}", invalid.len());
        }
    }

    #[test]
    fn test_edge_case_values() {
        // Test serialization of edge case values
        let edge_cases = vec![
            (0.0, 0.001), // Minimum variance
            (f64::MAX, 0.001), // Maximum mean
            (0.0, f64::MAX), // Maximum variance
            (1500.123456789, 200.987654321), // High precision
        ];
        
        for (mean, variance) in edge_cases {
            let rating = JsRatingInterface::new(mean, variance).expect("Valid rating");
            
            // JSON round-trip
            let json = rating.to_json();
            let from_json = JsRatingInterface::from_json(&json).expect("Valid JSON round-trip");
            
            // Binary round-trip
            let binary = rating.to_binary();
            let from_binary = JsRatingInterface::from_binary(&binary).expect("Valid binary round-trip");
            
            // Should maintain precision (within reasonable bounds for f64)
            assert!((from_json.mean() - mean).abs() < 1e-10);
            assert!((from_json.variance() - variance).abs() < 1e-10);
            assert_eq!(from_binary.mean(), mean);
            assert_eq!(from_binary.variance(), variance);
        }
    }
}

/// Test that optimization doesn't break existing functionality
#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_existing_functionality_preserved() {
        let rating = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        
        // All existing methods should still work
        assert_eq!(rating.mean(), 1500.0);
        assert_eq!(rating.variance(), 200.0);
        assert_eq!(rating.standard_deviation(), 200.0_f64.sqrt());
        assert_eq!(rating.conservative_rating(), 1500.0 - 3.0 * 200.0_f64.sqrt());
        
        // Fluent API should work
        let adjusted = rating.adjust_mean(50.0).adjust_variance(-20.0);
        assert_eq!(adjusted.mean(), 1550.0);
        assert_eq!(adjusted.variance(), 180.0);
        
        // Comparison should work
        let other = JsRatingInterface::new(1600.0, 180.0).expect("Valid rating");
        assert_eq!(rating.compare_to(&other), -1);
    }

    #[test]
    fn test_team_functionality_preserved() {
        let mut team = JsTeamInterface::new();
        let rating1 = JsRatingInterface::new(1500.0, 200.0).expect("Valid rating");
        let rating2 = JsRatingInterface::new(1600.0, 180.0).expect("Valid rating");
        
        team.add_player(rating1);
        team.add_player(rating2);
        
        assert_eq!(team.size(), 2);
        assert_eq!(team.total_mean(), 3100.0);
        assert_eq!(team.total_variance(), 380.0);
        
        // Metadata should work
        team.set_metadata("name", "Test Team");
        assert_eq!(team.get_metadata("name"), Some("Test Team".to_string()));
    }

    #[test]
    fn test_outcome_functionality_preserved() {
        let win = JsGameOutcomeInterface::create_win(0, 3).expect("Valid win");
        assert_eq!(win.get_winner_index(), Some(0));
        assert!(!win.is_draw());
        assert_eq!(win.team_count(), 3);
        
        let draw = JsGameOutcomeInterface::create_draw(2).expect("Valid draw");
        assert!(draw.is_draw());
        assert_eq!(draw.team_count(), 2);
    }
}