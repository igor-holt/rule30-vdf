# Rule 30 binary-tower AIR (bind, do not replace)

Status: specification only. `src/air.rs` remains the Winterfell prime-field AIR.
Asset: `evt_rule30_asic_binary_tower_20261009` (80/20 Knowledge Engine v2).
Linear: DIA-29.

## What this does not do

- Does not replace `src/air.rs` or `src/trace.rs`.
- Does not claim the repository verifies a binary-tower SNARK.
- Does not claim Rule 30 sequentiality is proved.
- Does not claim ASIC cycle times have been measured. Published numbers are CPU proof times.

## Shipped machine (unchanged)

Winterfell over a 128-bit prime field. Width `N`, `2N` columns, `4N` degree-2 constraints:

- boolean on the cell: `x^2 - x`
- boolean on the OR witness: `w^2 - w`
- prime-field OR: `w - (x + x_right - x * x_right)`
- prime-field XOR: `y - (x_left + w - 2 * x_left * w)`

Correct only while values stay in `{0,1}`. The coefficient `2` is a characteristic artifact.

## Bound statement (characteristic 2)

On a periodic ring the Rule 30 step is

```
s'_i = s_{i-1} + s_i + s_{i+1} + s_i * s_{i+1}
```

Depth two: one AND, three XORs. Horizontal width `N` is parallel. Delay parameter is row count `T`.

Single constraint per cell, degree 2, no boolean patch, no witness column:

```
y_i + s_{i-1} + s_i + s_{i+1} + s_i * s_{i+1} = 0
```

ASIC saturation is a hardware thesis: further silicon spend hits gate and wire delay, not an algorithmic gap. Sequentiality is the computational-irreducibility conjecture on this ring, not Wolfram's single-seed half-line.

## Verifier target

Diamond and Posen, Succinct Arguments over Towers of Binary Fields, IACR ePrint 2023/1784. Implementation family: Binius / FRI-Binius. Commit the bit trace on the tower F2 < F4 < F16. Evaluation of `T` rows stays sequential. The SNARK prover may be parallel. A green proof attests the AIR and the endpoints. It does not attest irreducibility.

## Acceptance for a later implementation PR

1. New module, not an edit that deletes the Winterfell AIR.
2. Cross-check the tower trace against `rule30_plain`.
3. Public inputs remain init state, final state, `N`, `T`.
4. Proof must fail if any cell violates the degree-2 rule.
