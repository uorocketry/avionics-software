#![no_std]
//!
//! This crate contains code to convert strain gauge voltages to temperatures.
//!

/// Coefficients for strain gauge conversion
const GAUGE_FACTOR: f64 = 2.0; // Gauge factor for the strain gauge
const V_REF: f64 = 5.0; // Output voltage reference

/*
* Function to convert voltage to strain
*
* @param voltage: f64 - The voltage output from the strain gauge
* @param gauge_factor: f64 - The gauge factor of the strain gauge
* @return strain: f64 - The calculated strain value
 */
pub fn voltage_to_strain(voltage: f64, gauge_factor: f64) -> f64 {
    // Convert voltage to strain using the gauge factor and resistance
    let mut strain = (voltage) / ((0.25) * GAUGE_FACTOR * V_REF);
    // Apply correction factor
    if gauge_factor != 0.0 || !gauge_factor.is_nan() {
        strain = strain * (GAUGE_FACTOR / gauge_factor);
    }
    return strain;
}
