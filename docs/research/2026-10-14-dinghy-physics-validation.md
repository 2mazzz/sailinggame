# Dinghy physics validation (ILCA/Laser) against primary sources

Date: 2026-10-14. Scope: `crates/sailsim` as of the working tree today (`params.rs::ilca_like`, `aero.rs`, `foil.rs`, `hydro.rs`, `stability.rs`, `sim.rs`, `tests/`). No source was edited.

Sim numbers below were produced with a throwaway probe crate outside the repo (it depends on the sailsim sources by path; fixed heading, best sheet over a 0.05 grid, 90 s from rest unless stated). "kn" is knots, 1 kn = 0.5144 m/s. Numbers read off a published figure are marked "(fig.)" and are good to about +-0.1 kn.

## Summary table

| # | Topic | Sim now | Primary evidence | Verdict | Confidence |
|---|-------|---------|------------------|---------|------------|
| 1 | Particulars | L 4.2 m, 7 m2, 60+80 kg, board 0.15 m2, rudder 0.06 m2 | LOA 4.23 m, 7.06 m2, hull 58-59 kg, rig+foils ~22 kg, all-up 160 kg; rudder ~0.09 m2 (my estimate from class drawing) | Length, sail area, hull mass consistent. Displacement (140 kg) too low by 20 kg. Rudder area probably too low. Board area unknown. | High (sail, length, mass); low (foil areas) |
| 2a | Upwind speed, 12 kn | 5.4-5.5 kn at 45 deg TWA; best-VMG angle 44-48 deg | 5.0 kn at 42, 5.4 kn at 46.5, 5.6 kn at 50 deg (fig.); best angle 40-46 deg | Consistent | High |
| 2b | Upwind VMG vs wind | 1.35 kn at 5 kn, 2.3 at 8, 3.5 at 12, 4.8 at 18 | 2.3 at 5 kn, 3.25 at 8, 3.75 at 12, 3.8 at 16 (fig.) | Too low in light air (-25 to -40%), too high above 14 kn (+10 to +25%); no saturation | Medium |
| 2c | Beam reach, 12 kn | 7.9 kn (best trim) | about 6.5 kn (fig., measured) | Too high (+20%) | Medium |
| 2d | Dead downwind | 5.6 kn at 12 kn; 3.65 kn at 8 kn | 5.2 kn at 12 kn, 3.8 kn at 8 kn (VPP, fig.); measured 3.1 kn VMG at 3.5-8 kn wind | Consistent (within 7%) | Medium |
| 2e | Planing, no-go zone | No planing; sails at 35 deg TWA (4.1 kn at 12 kn), stalls at 30 | Planing onset about 6.5 kn boat speed; no primary data on 30-38 deg | Planing missing (known); no-go unknown | Medium / low |
| 3 | Sail CL/CD | CLmax 1.11 at 37 deg AoA; CL 0.8 at the upwind operating point | ORC low-lift CLmax 1.347 at 28 deg apparent; Laser wind tunnel 1.30-1.36 | CL too low by about 40% upwind; L/D about right | Medium |
| 4 | Hull resistance | 3.0 v + 6.5 v^2 | Tank test, bare hull, 160 kg: hump at 6.5 kn | Too high at 2-3 kn (+30-50%), right at 4 kn, too low at 5-9 kn (-25 to -40%) | High |
| 5a | Foil lift slope, stall | board 3.1, stall 14.3 deg; rudder 3.5, 17.2 deg | NACA 0012 stalls near 14 deg; finite-AR slope 2.5-4 per rad | Consistent | Medium |
| 5b | Leeway | 4-6.5 deg upwind | No primary figure found | Unknown | Low |
| 6a | Righting moment | GZ = 0.45 sin(pi h / 85 deg); 223 N m at 10 deg | Hiking pose 713 N m near upright (one subject) | Peak magnitude plausible, shape wrong: far too weak at small heel | Medium |
| 6b | Heel at 12 kn | 20 deg | about 2.5 deg (VPP, fig.); flat up to 9.5 kn | Too high by about 8x | Medium |
| 6c | Vanishing/capsize angle, capsize wind | 85 deg, 70 deg, 19 kn | Not found | Unknown | n/a |
| 7a | Weather helm | CE 0.75 m aft of CLR (about 20% LWL); steady helm 0 deg | Not found (only that moderate helm helps by about 1%) | Unknown | Low |
| 7b | Turn rate, tack | 26 deg/s at 2 m/s full rudder; tack 6 s; min speed 44% | Not found | Unknown | n/a |
| 7c | Irons recovery | 22 s to fall off 45 deg, no recovery within 60 s | Project target 5-10 s | Fails own target | High (sim fact) |

## Prioritised changes

**Status (ticket 14, resolved):** items 1, 2 and 3 are done together: the hiking righting curve, the tank-data hull resistance table, depowering and a stronger sail (see the design doc, Known model limits, for the numbers). Item 4 was done in ticket 13, item 5 partly. Item 6 (irons) was done in ticket 3 with a sculling moment, not a back-winded sail. Not resolved: light-air VMG is still about 20 % low, and the beam-reach target needed a lower sail lift ceiling (1.15) than the ORC table (1.35) because the ORC lift on the reach plus the tank resistance gives about 7.9 kn against the measured 6.5.

**Status note (2026-10-18):** Q2e (planing) and Q5 (rudder and foil) were revisited on 2026-10-18. See [planing and rudder sources](2026-10-18-planing-and-rudder-sources.md) and the [parameter audit](2026-10-18-parameter-audit.md). The findings in this document are left as written.

1. Replace the sine GZ with a hiking-crew curve (`stability.rs`). It is the main reason for 20 deg heel at 12 kn and it distorts the sail balance (Q6).
2. Fix the resistance curve: remove the linear term, add a speed-dependent coefficient with a hump at 5-6.5 kn, or tabulate the Day and Nixon drag area (Q4).
3. Add proper depowering (flat/spill or an RM-limited sheet) so upwind VMG saturates at 3.8 kn above 12 kn; raise sail CL toward the ORC low-lift curve (Q2, Q3). The two errors currently cancel at 12 kn only.
4. Set `displacement_mass` to 160 kg and `surge_mass` to about 170 kg (Q1).
5. Rudder area 0.06 -> about 0.09 m2; re-check board area and whether it should include hull lateral area (Q1, Q5).
6. Fix irons recovery (back-winded sail force and reverse rudder at negative surge) (Q7).
7. Get primary data for leeway, helm angle and tack time before tuning them (Q5, Q7).

Caution from scratch trials (not committed): param-only changes cannot fix this. With the hiking GZ plus a stronger sail, 12 kn upwind VMG rose to 4.1-4.8 kn (target 3.75) because the sim has no depowering. Items 1-3 must be done together.

---

## 1. Particulars

Sim (`params.rs`): `surge_mass` 150, `displacement_mass` 140 (60 hull + 80 crew), `sail_area` 7.0, `board_area` 0.15 (comment says it includes hull and rudder lateral area), `rudder_area` 0.06, `board` AR 2.0, `rudder` AR 2.5. Design doc: length 4.2 m, beam 1.4 m (neither is a parameter).

Sources:
- Length, beam, hull mass, sail area. Wikipedia infobox (secondary): LOA 4.23 m, LWL 3.81 m, beam 1.37 m, hull 58.97 kg, mainsail 7.06 m2, draft 0.787 m. Corroborated by Taylor et al. (2016): "LWL = 3.81m, BWL = 1.37m ... SA = 7.06m2" (they label 1.37 m as BWL; Day and Nixon measured Bwl 1.10 m, so 1.37 m is the overall beam) and by Day and Nixon (2014): "The nominal hull weight including fittings is quoted by the manufacturer as 58kg" plus "mast, boom, foils, rudder stock, tiller ... weighed at approximately 22kg", benchmark 160 kg all-up with an 80 kg sailor, Lwl 3.79 m at 160 kg. A Laser paper in PMC (Performance Factors in Dinghy Sailing) gives beam 1.42 m, so beam varies by source (1.37-1.42 m).
- Foils (ILCA Class Rules 2020, measurement diagram, mm): rudder blade maximum thickness 20 (rule 15(k): blade plus padding "must not exceed 20.3mm"), leading edge max 635, trailing edge max 527, width 203; centreboard leading edge max 680, width 341 at mid-height, thickness max 33. The rules give outline maxima only, no area. My estimate (not a source figure): exposed centreboard about 0.68 m x 0.30 m = about 0.2 m2; submerged rudder about 0.45 m x 0.2 m = about 0.09 m2.
- Standard-rig sailor: Day (2017) uses 80 kg, 1.83 m and cites Goodison (2008) for 78-83 kg as optimum.

Verdict: length, sail area, hull mass consistent (high confidence). Displacement 140 kg is too low: the tank-tested and VPP-modelled all-up mass is 160 kg because the rig and foils add about 22 kg. Rudder area 0.06 m2 is probably 30% low (low confidence, estimate from drawing). Board area 0.15 m2 is plausible only if it is meant to be net of the hull; unknown.

Change: `displacement_mass` 160.0, `surge_mass` 170.0 (160 plus about 10 added mass), `rudder_area` 0.09. Heel and yaw inertia should scale with the extra rig mass (rig is high, so `heel_inertia` rises more than 14%).

## 2. Speed

Sim (probe, 12 kn = 6.17 m/s true wind, fixed heading):

| TWA | speed (kn) | leeway (deg) | heel (deg) |
|-----|-----------|--------------|------------|
| 35 | 4.1 | 6.5 | 14.5 |
| 40 | 4.9 | 5.7 | 18.5 |
| 45 | 5.4 | 4.7 | 19.7 |
| 50 | 5.9 | 4.0 | 20.4 |
| 60 | 6.7 | 3.1 | 20.8 |
| 90 | 7.9 | - | 16.8 |
| 120 | 7.35 | - | 7.1 |
| 150 | 6.2 | - | 2.4 |
| 180 | 5.6 | - | 1.0 |

Best upwind VMG over TWA 34-60: 1.35 / 1.67 / 2.31 / 2.93 / 3.51 / 4.02 / 4.43 / 4.77 kn at 5 / 6 / 8 / 10 / 12 / 14 / 16 / 18 kn, best angle 42-48 deg. Beam reach at 6 m/s: 3.59 m/s (7.0 kn) at sheet 0.5, best 3.97 m/s at sheet 0.6-0.7 (the brief's 3.3 m/s was not reproduced). No planing. TWA 30: 0 kn.

Sources:
- Day (2017), VPP validated against measurements (Binns et al. 2002, 80 kg crew): "A Laser can be expected to achieve maximum VMG upwind in true wind angles in the region around 40-46 degrees in flat water." Upwind speed at 12 kn (Fig. 5): about 4.9 kn at 40 deg, 5.0 at 42, 5.4 at 46.5, 5.6 at 50, 6.3 at 60 deg; "speed is relatively insensitive to the wind speed". Upwind VMG (Fig. 8): about 2.3 kn at 5 kn wind, 3.25 at 8, 3.6 at 10, 3.75 at 12, 3.8 at 14-16. Downwind VMG (Fig. 8): about 2.5 at 5 kn, 3.8 at 8, 5.2 at 12, 6.7 at 16. Apparent wind angle upwind "between around 27 degrees in 5 knots of wind up to around 33 degrees in 15 knots". Measured Binns et al. polar at 12 kn (Fig. 3, read off, +-0.3 kn): about 4.7 kn at 37 deg, about 6.5 kn near 90 deg, about 6.6 kn dead downwind (measured where "some surfing downwind could be expected"). Dead downwind at 9 kn: measured about 5.6 kn vs VPP 4.2-4.4 kn (Fig. 6).
- Caraballo et al. (2021), 159 Laser sailors, World Cup Genoa 2019, wind 3.5-8.3 kn (women) and 4.1-8 kn (men): mean VMG upwind 2.3 kn, downwind 2.9-3.2 kn, broad reach 4.1-4.7 kn (Table 1). Course-averaged, so a lower bound on steady VMG.
- Day and Nixon (2014): drag area has "a major hump ... at around 6.5 knots", where "the boat starts to plane". Froude number there is 0.55 (3.34 m/s, LWL 3.81 m).

Verdicts:
- 12 kn, 40-50 deg TWA: sim 4.9 / 5.4 / 5.9 kn vs 4.9 / 5.3 / 5.6 kn. Consistent (within 5%). Upwind best angle 44-48 deg vs 40-46 deg: consistent, slightly wide. High confidence. The design doc's "optimistic close-hauled" remark is not supported at 12 kn by this evidence.
- Wind dependence is wrong. Sim VMG rises almost linearly with wind (1.35 to 4.8 kn from 5 to 18 kn); the VPP saturates at about 3.8 kn above 12 kn because the sailor is righting-moment limited and depowers. Light air -25 to -40%; 16-18 kn +17 to +25%. Medium confidence (figure reading).
- Beam reach 7.9 kn vs about 6.5 kn measured: too high by about 20%. The hull-resistance hump (Q4) is the likely cause. Medium confidence.
- Downwind: consistent with the VPP. The measured data are 0.5-1.5 kn faster, attributed by the authors to surfing and measurement error, so no change is justified.
- No planing: onset is about 6.5 kn boat speed (high confidence), so beam and broad reaches above 12 kn will be wrong. Known limit, out of milestone-1 scope.
- 30-35 deg TWA: no primary data found; the sim sails 4.1 kn at 35 deg but 0 at 30 deg. Unknown.

Change: see Q3, Q4 and Q6. Speed errors come from the resistance hump, the weak sail and the missing hiking limit, not from one parameter.

## 3. Sail aerodynamics

Sim (`aero.rs`, `foil.rs`, `params.sail`): `lift_slope` 4.5/rad, `stall_angle` 0.35 rad (20 deg), `drag_zero_lift` 0.04, `aspect_ratio` 3.0, `plate_normal` 1.2. AoA = apparent angle minus boom angle; below zero there is no lift. Blend: `s = x^2/(1+x^2)`, `x = alpha/stall`, so lift is already 50% stalled at 20 deg. CL(alpha): 0.38 at 5 deg, 0.67 at 10, 0.86 at 15, 0.98 at 20, 1.09 at 30, 1.10 at 40, 0.94 at 60, 0.33 at 90. CLmax 1.11 at 37 deg. CD: 0.06, 0.12, 0.20, 0.30, 0.51 (30 deg), 1.19 (60 deg), 1.54 (90 deg). The attached-flow induced term is not removed from the blend, so it adds 0.36 to CD at 90 deg.

At the 12 kn, 45 deg TWA operating point (sheet 0.85, boom 17.75 deg): apparent wind angle about 31 deg, AoA about 14 deg, CL about 0.8, CD about 0.17 (L/D about 4.7).

Sources:
- ORC "low lift" mainsail table (Day 2017, Table 1, coefficients on cloth area, as a function of apparent wind angle with a trimmed sail): CL 0.862 at 7 deg, 1.164 at 12 deg, **1.347 at 28 deg**, 1.239 at 60 deg, 1.125 at 90 deg, 0.838 at 120 deg, -0.112 at 180 deg; viscous CD 0.033 at 28 deg, 0.113 at 60 deg, 0.383 at 90 deg, 1.345 at 180 deg. Lift-related drag is added from an effective aspect ratio plus a separation term of 0.005.
- Flay (1992) wind tunnel, 1/6 Laser rig (via Day 2017): lift peaks "around 1.53 in turbulent flow" at 28.5 deg, which is 1.30-1.36 after correcting to cloth area. Total drag coefficient about 0.27 including spars and hull, so L/D about 5. Day concludes "the ORC 'low lift' coefficients are appropriate for a Laser".
- Day (2017): raising CD at 180 deg from 1.345 to 1.5 changed predicted speed by "just over 0.1 knots".

Verdict: Too low upwind. The sim sail makes about 0.8 of CL at its upwind operating point against about 1.3 for the real sail (-40%), and its CLmax is 1.11 against 1.35. The stall blend starts too early. The L/D is similar, so the sail is a weaker version of the real one. Downwind drag is consistent (1.54 vs 1.345 at 90 deg AoA, small effect per Day). Medium confidence: the ORC table is indexed by apparent angle, so the comparison uses my operating-point estimate of AoA. The sim speeds are right at 12 kn only because hull resistance is too low (Q4), which cancels this error.

Change (to be tuned together with Q4 and Q6): `sail.stall_angle` 0.35 -> 0.45, `sail.lift_slope` 4.5 -> about 5.2, `sail.aspect_ratio` 3 -> 4.5; and in `foil.rs` use a steeper blend (for example `x^4/(1+x^4)`) and multiply the induced term by `(1 - stalled)` only where it is not already, so CD at 90 deg is about 1.2. Scratch trial: stall 0.45 alone gives +4-6% VMG at 8-12 kn; slope 5.5/stall 0.45/AR 5 gives 12 kn VMG 4.2 kn, too high without a depowering model.

## 4. Hull resistance

Sim (`hydro.rs`): R = 3.0 |u| + 6.5 u^2 (N), hull only; board and rudder drag are added separately.

Source (Day and Nixon 2014, tank tests, Laser hull without appendages, 160 kg, level trim, fresh water; drag area = R / (0.5 rho V^2), Fig. 2 of Day 2017, read off): 0.0125 m2 at 2 kn, 0.0132 at 3, 0.0157 at 4, 0.0207 at 5, 0.0245 at 6.5, 0.0221 at 8, 0.0206 at 9. "a major hump in drag area at around 6.5 knots ... a small hump can also be seen around 3.5 knots". Delft residuary regression under-predicts the 3-5 kn range by up to 8% and the weight sensitivity; "resistance regression formulae derived for sailing yachts should be treated with some caution". 10 kg more mass changes resistance by 2.7-6.2%. Hull speed 1.34 sqrt(LWL ft) with LWL 3.81 m = 12.5 ft gives 4.7 kn (Froude 0.4), a rule of thumb.

Converted (my arithmetic, R = 0.5 x 999 x V^2 x drag area):

| speed | tank hull R (N) | sim hull R (N) | sim/tank |
|-------|-----------------|----------------|----------|
| 2 kn | 6.6 | 10.0 | 1.5 |
| 3 kn | 15.7 | 20.1 | 1.28 |
| 4 kn | 33 | 33.7 | 1.0 |
| 5 kn | 68 | 50.7 | 0.74 |
| 6.5 kn | 137 | 82.7 | 0.60 |
| 8 kn | 187 | 122 | 0.65 |
| 9 kn | 220 | 153 | 0.70 |

Verdict: the sim matches at 4 kn only. It is too high below 3 kn (the linear term) and 25-40% too low from 5 to 9 kn, and it has no hump or planing drop. Day's VPP adds about 20% of hydrodynamic drag upwind (14% induced) on top of the bare hull; the sim's board model supplies part of that. High confidence for the hull curve (measured, one hull, level trim).

Change: `resistance_linear` 0.0. Quadratic coefficient must vary with speed; simplest is a table (kn: drag area) from Day and Nixon interpolated in `hull_resistance`. If a polynomial is needed: k = 6.2 below 3 kn, rising to about 11-12 (R = k v^2) at 5-8 kn, 10.4 at 5 kn and 12.3 at 6.5 kn. Scratch trial: `resistance_linear` 0, `resistance_quadratic` 6.2 raised 5-16 kn VMG by 11-18%, so the hump will need a compensating sail change.

## 5. Foils and leeway

Sim: `board` slope 3.1, stall 0.25 rad (14.3 deg), CD0 0.012, AR 2.0, plate_normal 1.1; `rudder` slope 3.5, stall 0.30 rad (17.2 deg), AR 2.5. With the blend, board CL is 0.43 at 10 deg and peaks at 0.73 at 42 deg; rudder CL is 0.50 at 10 deg, peak 0.84.

Sources:
- NACA TN 3361 (Critzos et al. 1955), NACA 0012 at Re 1.8e6: stall at "alpha = 14 deg"; maximum lift coefficient 1.33 smooth and 1.07 with roughness; section drag at 90 deg 2.08 (Re 1.8e6) and 1.95 (Re 0.5e6).
- Sandia SAND80-2114 (Sheldahl and Klimas 1981), NACA 0012 at Re 0.36-0.70e6 (dinghy foil Reynolds numbers are about 4-8e5 for 0.25 m chord at 2-3 m/s): "cl max has increased to 1.0 for positive angles". 
- Day (2017, Keuning and Verwerft method): dCL/dalpha = 5.7 AR_E / (1.8 + cos(L) sqrt(AR_E^2/cos^4(L) + 4)) with AR_E = 2 b / c (free-surface doubling). For AR_E = 2, 3, 4, 5 this gives 2.46, 3.1, 3.64, 3.96 per rad (my arithmetic). A Laser board of about 0.68 m span and 0.29 m mean chord has AR_E about 4.7, about 3.9 per rad, plus a hull carry-over factor of 1 + 1.8 Tc/b (Tc 0.094 m from Day and Nixon), about 1.25. The effective alpha is reduced by a zero-lift drift angle and by heel (factor 1 - 0.382 heel).
- Leeway: not found in a primary source. Day (2017) solves leeway but quotes no values. A secondary search snippet for a 470 (SNAME JST 2020, not opened) mentions about 5 deg; treat as unverified.

Verdict: slopes (3.1 and 3.5) lie in the published range (2.5-4 per rad for AR_E 2-4.5): consistent. Stall angles consistent with NACA 0012 (14 deg). The sim CL(10 deg) is 20% below its own attached-flow value because the blend starts early; with the weak sail this partly explains leeway of 4-6.5 deg. Leeway is unknown (low confidence). Rudder area, not slope, is the likelier issue (Q1). `plate_normal` 1.1 is a finite-plate value, below the 2-D 2.0 of the section tests; acceptable for AR about 2 but unverified here.

Change: none required. If leeway is later measured, tune `board_area` (0.15 -> 0.2 m2) before the slope.

## 6. Heel and stability

Sim (`stability.rs`): GZ = 0.45 sin(pi heel / 85 deg) x 140 kg x 9.81; peak 618 N m at 42.5 deg, zero at 85 deg. RM: 114 N m at 5 deg, 223 at 10, 416 at 20, 553 at 30, 618 at 42.5, 493 at 60, 325 at 70. `capsize_heel` 70 deg. Heeling arm 2.0 m, sail force reduced by cos(heel). Probe: heel at 45 deg TWA, sheet 0.85 is 3.3 deg at 5 kn, 8.5 at 8, 13.4 at 10, 19.7 at 12, 32.5 at 15, 56.6 at 18; capsize (sheet 0.85) at 19 kn. Heel at 12 kn with best trim: 20 deg.

Sources:
- Day (2017): "the freedom of the crew to move around the boat essentially decouples the heel angle from the available righting moment"; maximum RM computed with the crew CG up to 95% of its standing CG height (55% of height, 1.83 m crew gives about 0.96 m) from the centreline: RM_max = W_hull GZ(phi) + W_crew [GZ(phi) + dY cos(phi) - dZ sin(phi)]. Boat without sail weighed 79.5 kg, CG 12 cm above deck. Results (Fig. 9): heel 0 deg upwind "up to about 9 knots", about 2.5 deg at 12 kn and about 3 deg at 16 kn; "The VPP predicts that the boat should be sailed flat upwind up to about 9 knots wind speed, and then at a small heel angle".
- Taylor et al. (2016), laboratory on a flat board, one 75 kg 1.8 m subject: measured hiking moments about the ankles 713, 525 and 57 N m for poses 1, 3, 2; pose capture within 5%. These are single-subject, static, and not defined as a maximum.
- Max RM near upright is therefore about 700 N m, equivalent to GZ about 0.5 m on 140 kg (my arithmetic). It falls with cos(heel), not with a sine that starts at zero.
- Vanishing-stability angle, capsize angle and capsize wind speed for a Laser: not found in a primary source.

Verdict:
- Peak GZ 0.45 m is the right order of magnitude (consistent, medium confidence).
- The curve shape is wrong: with a hiking sailor the moment is largest near upright. The sim gives 223 N m at 10 deg against about 700 N m available, so it heels 20 deg at 12 kn where real boats are at about 2.5 deg. Too high, medium confidence (heel read off Fig. 9).
- 85 deg, 70 deg and 19 kn: unknown. Once hiking RM is fixed the capsize wind speed will rise sharply; the test `sheeted_in_close_hauled_in_a_gale_capsizes` (12 m/s = 23 kn) should still pass but check.

Change (structural, `stability.rs`): `gz = arm * tanh(heel / 0.035) * cos(pi/2 * |heel| / vanishing_heel)` with `righting_arm_max` 0.52 and `displacement_mass` 160 (RM about 800 N m upright). The tanh keeps it odd and smooth at zero. Scratch trial with 0.52 and mass 140: heel at 12 kn drops to 1.5 deg, 16 kn 4.8 deg. This needs the depowering model of Q3 to avoid the speed overshoot above. Longer term, make crew hiking an action or a controller.

## 7. Weather helm, turning, tacking, irons

Sim: `board_arm` 0.5 m forward of CG, `sail_ce_aft` 0.25 m aft of CG, so the sail CE is 0.75 m aft of the board centre of resistance (about 20% of LWL 3.81 m; the conventional "lead" has the CE ahead of the CLR, so the sign is opposite). Yaw: `yaw_inertia` 200, damping 80 + 60 |r|, `rudder_arm` 1.7 m, `rudder_max_angle` 35 deg. Probe results:
- Steady helm to hold 45 deg TWA at 8 and 12 kn: about 0.1 deg of blade. The board and the rudder acting as a fin cancel the CE offset.
- Full rudder, no wind: peak yaw rate 11.5 / 18.8 / 26 / 34 / 41 / 56 deg/s at 1 / 1.5 / 2 / 2.5 / 3 / 4 m/s; 90 deg of turn takes 8.1 s at 1.5 m/s, 5.5 s at 2 m/s, 4.1 s at 2.5 m/s.
- Tack (12 kn, 45 to -45 deg, the test helmsman, sheet 0.85): head to wind at 1.5 s, new heading within 5 deg at 6.0 s, minimum speed 2.35 kn = 44% of 5.4 kn, 90% speed again at 11.6 s after the start.
- Irons (head to wind, 6 m/s, speed 0, full rudder either way, sheet 0.5): backs at 0.41 m/s and takes 22 s to fall off 45 deg; speed never reaches 1.5 m/s in 60 s.

Sources:
- Day (2017): "moderate weather helm can improve performance by differentially loading the rudder and unloading the hull/keel", worth about 1.0% upwind speed; mast bend moves the CE aft (equivalent rake 11 deg); sail CE 36.4% of luff above the boom and 33% of the foot aft of the luff. No helm angle quoted. Hull Munk moment is modelled and destabilising.
- Lead as a percentage of LWL: only forum posts quoting ranges (below 10% to over 17%, mainly for keelboats) were found. Larsson and Eliasson, "Principles of Yacht Design", was not accessible: "not found".
- Turn rates, tack duration, speed loss in a tack, and time to leave irons for a dinghy: no primary data found. A dinghy tacking model "has been neglected" in the literature per a search summary (not verified). The ILCA coaching page on tacking has no numbers.

Verdicts:
- Weather helm: unknown (low confidence). The 0.75 m aft CE offset is cancelled by the foils, so the helm is about zero rather than a few degrees; do not tune without a reference value.
- Turn rate and tack: unknown. A 6 s tack and 44% minimum speed look plausible but are not supported by a source.
- Irons: the sim does not meet its own 5-10 s target (22 s plus no recovery). This is a model gap: a luffing sail makes no force (`alpha <= 0`), so there is no back-winded sail pushing the bow round, and the rudder at negative surge is not modelled to steer in reverse.

Change: add a back-winded sail force (alpha < 0 gives lift of the opposite sign with a flat-plate coefficient) and let the rudder act on stern flow when surge is negative. Re-test with `tests/steering.rs` extended by an irons case.

---

## Sources

- ILCA. ILCA Class Rules 2020, including Measurement Diagrams (pp. 37-43), World Sailing document 26057. https://sailing.org/tools/documents/2020ILCAClassRules-[26057].pdf. Used for foil outline dimensions, rudder thickness limit, standard-rig sail dimensions (luff 5.12 m, foot 2.75 m, leech 5.555 m for the MkII sail). Hull mass and sail area are not stated in this document.
- Day, A.H. (2017). Performance prediction for sailing dinghies. Ocean Engineering (2nd revision, Feb 2017). https://strathprints.strath.ac.uk/59980/1/Day_OE_2017_Performance_prediction_for_sailing_dinghies.pdf (ScienceDirect PII S0029801817300938). Used for ORC low-lift table, VPP results, heel, RM method, foil formulas.
- Day, A.H. and Nixon, E. (2014). Measurement and prediction of the resistance of a Laser sailing dinghy. International Journal of Small Craft Technology (RINA Transactions). https://strathprints.strath.ac.uk/46407/7/Day_Nixon_IJSCT_2014_Measurement_and_prediction_of_the_resistance_of_a_laser_sailing.pdf
- Taylor, J.C., Banks, J. and Turnock, S.R. (2016). The influence of sailor position and motion on the performance prediction of racing dinghies. Proc. Chesapeake Sailing Yacht Symposium. https://eprints.soton.ac.uk/400462/1/CSYS_202016_20Taylor_20Revised_20V8.pdf
- Caraballo, I., Conde-Caveda, J., Pezelj, L., Milavic, B. and Castro-Pinero, J. (2021). GNSS Applications to Assess Performance in Olympic Sailors: Laser Class. Applied Sciences 11(1), 264. https://doi.org/10.3390/app11010264 (full text via https://rodin.uca.es/handle/10498/24448).
- Critzos, C.C., Heyson, H.H. and Boswinkle, R.W. Jr. (1955). Aerodynamic Characteristics of NACA 0012 Airfoil Section at Angles of Attack from 0 to 180 Degrees. NACA TN 3361. https://ntrs.nasa.gov/api/citations/19930084501/downloads/19930084501.pdf
- Sheldahl, R.E. and Klimas, P.C. (1981). Aerodynamic Characteristics of Seven Symmetrical Airfoil Sections Through 180-Degree Angle of Attack for Use in Aerodynamic Analysis of Vertical Axis Wind Turbines. Sandia National Laboratories, SAND80-2114. https://digital.library.unt.edu/ark:/67531/metadc1191254/
- Cited through Day (2017), not read directly: Offshore Racing Congress (2013) ORC VPP Documentation (current versions: https://orc2.orc.org/rules/ORC%20VPP%20documentation%202022.pdf); Binns et al. (2002) Laser sailing simulator polars; Flay (1992) Laser rig wind tunnel; Keuning and Katgert (2008) and Keuning and Verwerft (2009) Delft appendage formulas; Kerwin (1978) VPP; Goodison (2008) crew weight; Clark (2014) instrumented Laser (the author of Day (2017) discards these data as inconsistent).
- Secondary, lower confidence: Wikipedia, "Laser (dinghy)", https://en.wikipedia.org/wiki/Laser_(dinghy) (LOA, LWL, beam, draft, hull mass, sail area); Performance Factors in Dinghy Sailing: Laser Class, PMC6950116 (beam 1.42 m); boatdesign.net forum threads on lead (range only, not relied on).

## Not found (explicitly)

ILCA-published board and rudder areas; hull mass from the class rules text (builder construction manual); leeway angle measurements; righting moment curve, angle of vanishing stability and capsize angle for a Laser; capsize wind speed; weather helm angle and lead as a percentage of LWL from a primary source; turning rate, tack duration, speed loss in a tack and irons recovery time for a dinghy; Marchaj and Larsson and Eliasson specifics (books not accessible).
