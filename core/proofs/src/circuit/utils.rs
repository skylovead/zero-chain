use bellman::{SynthesisError, ConstraintSystem};
use scrypto::circuit::{
    boolean::{Boolean, field_into_boolean_vec_le},
    ecc::{EdwardsPoint, fixed_base_multiplication},
};
use scrypto::jubjub::{JubjubEngine, FixedGenerators, edwards, PrimeOrder};
use crate::ProofGenerationKey;

/// This performs equal veficiation of two edward points.
pub fn eq_edwards_points<E, CS>(
    mut cs: CS,
    a: &EdwardsPoint<E>,
    b: &EdwardsPoint<E>,
) -> Result<(), SynthesisError>
where
    E: JubjubEngine,
    CS: ConstraintSystem<E>,
{
    let a_repr = a.repr(cs.namespace(|| "a into representation."))?;
    let b_repr = b.repr(cs.namespace(|| "b into representation."))?;

    for (i, (a, b)) in a_repr.iter().zip(b_repr.iter()).enumerate() {
        Boolean::enforce_equal(
            cs.namespace(|| format!("a_repr equals b_repr {}", i)),
            &a,
            &b
        )?;
    }

    Ok(())
}

/// Inputize re-randomized signature verification key.
pub fn rvk_inputize<E, CS>(
    mut cs: CS,
    proof_gen_key: Option<&ProofGenerationKey<E>>,
    alpha: Option<&E::Fs>,
    params: &E::Params
) -> Result<(), SynthesisError>
where
    E: JubjubEngine,
    CS: ConstraintSystem<E>,
{
    // Ensure pgk on the curve.
    let pgk = EdwardsPoint::witness(
        cs.namespace(|| "pgk"),
        proof_gen_key.as_ref().map(|k| k.0.clone()),
        params
    )?;

    // Ensure pgk is large order.
    pgk.assert_not_small_order(
        cs.namespace(|| "pgk not small order"),
        params
    )?;

    // Re-randomized parameter for pgk
    let alpha = field_into_boolean_vec_le(
        cs.namespace(|| "alpha"),
        alpha.map(|e| *e)
    )?;

    // Make the alpha on the curve
    let alpha_g = fixed_base_multiplication(
        cs.namespace(|| "computation of randomiation for the signing key"),
        FixedGenerators::NoteCommitmentRandomness,
        &alpha,
        params
    )?;

    // Ensure re-randomaized sig-verification key is computed by the addition of ak and alpha_g
    let rvk = pgk.add(
        cs.namespace(|| "computation of rvk"),
        &alpha_g,
        params
    )?;

    // Ensure rvk is large order.
    rvk.assert_not_small_order(
        cs.namespace(|| "rvk not small order"),
        params
    )?;

    rvk.inputize(cs.namespace(|| "rvk"))?;

    Ok(())
}

pub fn g_epoch_nonce_inputize<E, CS>(
    mut cs: CS,
    g_epoch: Option<&edwards::Point<E, PrimeOrder>>,
    dec_key_bits: &[Boolean],
    params: &E::Params,
) -> Result<(), SynthesisError>
where
    E: JubjubEngine,
    CS: ConstraintSystem<E>,
{
    // Ensure g_epoch is on the curve.
    let g_epoch = EdwardsPoint::witness(
        cs.namespace(|| "g_epoch"),
        g_epoch.map(|e| e.clone()),
        params
    )?;

    // Ensure that nonce = dec_key * g_epoch
    let nonce = g_epoch.mul(
        cs.namespace(|| format!("g_epoch mul by dec_key")),
        dec_key_bits,
        params
    )?;

    g_epoch.inputize(cs.namespace(|| "inputize g_epoch"))?;
    nonce.inputize(cs.namespace(|| "inputize nonce"))?;

    Ok(())
}
