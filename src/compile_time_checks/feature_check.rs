// --- Mutually Exclusive Environment Features ---

#[cfg(all(feature = "prd", feature = "stg"))]
compile_error!("Features `prd` and `stg` cannot be enabled together.");

#[cfg(all(feature = "prd", feature = "localhost"))]
compile_error!("Features `prd` and `localhost` cannot be enabled together.");

#[cfg(all(feature = "stg", feature = "localhost"))]
compile_error!("Features `stg` and `localhost` cannot be enabled together.");

// #[cfg(not(any(feature = "prd", feature = "stg", feature = "localhost")))]
// compile_error!("You must enable exactly one of: prd, stg or localhost.");
