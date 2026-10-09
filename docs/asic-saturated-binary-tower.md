# Rule 30 as an ASIC-saturated state machine, verified on a binary tower

Status: approved next step. Not implemented in `src/air.rs`.

The shipped prover is Winterfell over a 128-bit prime field. Width `N`, `2N` columns, `4N` degree-2 constraints. Prime-field OR and XOR use a coefficient `2` and boolean patches `x^2 - x`. That encoding means Rule 30 only while every value stays in `{0,1}`.

## Native transition

On a periodic ring, characteristic 2:

```
s'_i = s_{i-1} + s_i + s_{i+1} + s_i * s_{i+1}
```

Left XOR (center OR right). Depth two: one AND, three XORs. No witness column.

Collapsed AIR, one constraint per cell:

```
y_i + s_{i-1} + s_i + s_{i+1} + s_i * s_{i+1} = 0
```

## What is sequential, what is not

Horizontal width `N` is parallel. The delay parameter is the row count `T`. Evaluation of the `T` rows stays sequential. The SNARK prover may be parallel. A green proof attests the AIR and the endpoints. It does not attest computational irreducibility.

ASIC saturation is a hardware thesis: the cell is already depth two, so further silicon spend hits gate and wire delay rather than an algorithmic gap. This repository has CPU proof times only. No ASIC cycle measurement is claimed.

The periodic ring is not Wolfram's single-seed half-line.

## Verifier target

Binary-tower SNARK (Diamond and Posen, ePrint 2023/1784; Binius / FRI-Binius). Commit the bit trace on the tower `F2 < F4 < F16 < ...` instead of inflating each bit into a 128-bit field element.

Do not wire this into the Winterfell path. `src/air.rs` remains the prime-field machine until a tower prover is actually bound.

Asset: `evt_rule30_asic_binary_tower_20261009`.
