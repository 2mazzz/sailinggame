# Source audit of `BoatParams::ilca_like()` (2026-10-18)

Scope: the numbers in `crates/sailsim/src/params.rs` that the earlier notes (`2026-10-14-dinghy-physics-validation.md`, `2026-10-18-planing-and-rudder-sources.md`) did not already source. Findings already in those notes are not repeated; this file only records what was newly checked. "Day 2017" is Day, Ocean Engineering (2017), "Performance prediction for sailing dinghies"; "Day and Nixon 2014" is the Laser tank-test paper. Confidence is mine. Arithmetic marked "mine" is not from a source.

Rule used for action: a value is changed only for medium or better evidence that points at a different value. Everything else is kept and flagged.

## Result in one paragraph

Of the parameters audited, medium-or-better sources exist for: the rudder planform efficiency (new, formula verified in the original paper; value changed 0.9 to 0.97), the 160 kg masses and the hull table (already in the earlier notes), and the sail CE height above the boom (1.864 m, but the heel arm needs inputs I could not source). `rudder_arm` is medium-low. Only one value changed. Everything else (inertias, damping, added mass, sculling, low-speed damping, depowering constants, boom angle, board arm, foil areas) has no primary source and stays flagged.

## Audit table

| parameter | current value | source | what the source says | confidence | action |
|---|---|---|---|---|---|
| `boom_angle_in` | 21 deg | ILCA Class Rules 2020, measurement diagrams (boom: max length 2740 mm, mainsheet fittings at 1186 and 1653 mm from the aft end; traveller and boom blocks, rules 3(c), 18). Day (2017) sec. 6.3 (boom beyond 90 deg when eased) | The rules fix fitting positions but give no sheeted-in boom angle. Day mentions only that the boom is sheeted out near 90 deg downwind (supports `boom_angle_eased` 90 deg). No source for the closest boom angle. Could be derived from the sheet geometry (traveller position and boom block spacing) but the traveller position is not in the text I extracted. | none | flagged (already noted "tuned, no source, ticket 4") |
| `sail_heel_arm` | 2.0 m | Day (2017) sec. 2.1: CE is 36.4 % of the luff above the boom. ILCA rules 2020: standard luff 5120 mm max | CE height above boom = 0.364 x 5.12 = 1.864 m (mine). The arm also needs the boom height above the waterline and the depth of the centreboard's centre of lateral resistance below it. Boom height above deck and deck height above water are not in the class rules text or any source I found. With plausible values (boom 0.4 to 0.5 m above deck, deck 0.3 m above water, board CLR 0.3 to 0.4 m below water) the arm would be 2.9 to 3.2 m, but these inputs are my guesses. Day (2017) sec. 2.3 also says twist lowers the CE to 32 % of the luff (1.64 m) at 15 kn. | low (only the 1.864 m is sourced; the offset is a guess that points above 2.0) | kept, flagged. Open question 1 |
| `sail_ce_aft` | 0.25 m aft of CG | Day (2017) sec. 2.1: longitudinal CE at 33 % of the foot aft of the luff; foot 2.75 m (ILCA 2020) | CE is 0.33 x 2.75 = 0.91 m aft of the luff (mine). The mast position relative to the boat's CG is not in any source I read, so the 0.25 m cannot be checked. Day adds that mast bend moves the CE aft (equivalent rake 11 deg), which would increase the lead. | low (sail-relative value is sourced, the boat-relative one is not) | kept, flagged. Open question 2 |
| `hike_arm_extra` | 0.15 m (RETUNED, no source) | Day (2017) sec. 4: maximum RM with the crew CG at 95 % of its standing CG height (about 0.96 m for a 1.83 m, 80 kg sailor). Taylor et al. (2016): 713, 525, 57 N m hiking moments in three poses (one 75 kg subject, ankle moment on a flat board) | Day's fully-extended crew gives about 80 x 9.81 x 0.96 = 750 N m upright (mine). The sim gives 160 x 9.81 x 0.45 = 706 N m at `hike` 0 and 942 N m at `hike` 1. The sourced 750 to 800 N m therefore matches `hike` 0 better than `hike` 1, which would suggest an extra of about 0.03 m. But the sim's `hike` 0 means "sitting on the gunwale", which is not Taylor's or Day's pose, and Taylor's 713 is one subject, one pose, an ankle moment. Tried as a what-if: 0.03 m fails `tests/hiking.rs` (`hiking_lowers_the_heel_in_the_same_wind`, `hiking_survives_a_wind_that_capsizes_a_sailor_sitting_in`) because the hike control then has no visible effect; beam-reach speeds unchanged (4.62, 6.51, 8.88, 11.71, 15.03). | low (the sources bound the upright moment, not the sit-in versus lean-out split) | kept. Not committed. Open question 3 |
| `depower_moment` | 450 N m (RETUNED) | Day (2017) sec. 2.3, 5, Fig. 8 and 9 | The VPP does not saturate a moment. It depowers with three variables: FLAT (lift multiplier, floored at 0.6), TWIST (lowers CE height, t = 0.186 gives CE at 32 % at 15 kn; induced drag scaled by 1 / (1 + 8 t^2)) and SPILL (reduces the angle of attack for the coefficients, used once FLAT is at its floor). It is solved per wind speed so the sailor's max RM is just balanced. Saturation of VMG at about 3.8 kn above 12 kn (Fig. 8) is a result of that solve, not an input. The sim's single `depower_moment` is a one-parameter stand-in with no direct counterpart. | none | kept, flagged |
| `depower_floor` | 0.35 | Day (2017) sec. 2.3 | FLAT cannot go below 0.6 (a 40 % lift cut, from Fossati et al. 2006 via Day). Further lift reduction is modelled as spill, which cuts the angle of attack, so the combined floor depends on the wind speed and has no fixed value. 0.35 is a plausible combined FLAT x SPILL x twist floor but is not sourced. | none (0.6 is the sourced FLAT-only limit) | kept, flagged |
| `board_arm` | 0.75 m (RETUNED) | Day (2017) sec. 3.3 (Keuning and Vermeulen 2003): CP of board and rudder on the quarter chord line at 43 % of the foil draft; plus hull Munk moment | Gives how to locate the CLR but not where the Laser board sits relative to its CG. No Laser board position, CG position or weather-helm angle found. Day (2017) sec. 6.3: moderate weather helm worth about 1 % speed, no angle. The 2026-10-14 note already marks 0.75 m of CE lead (about 20 % LWL) as too large against a yacht-design range of under 10 % to over 17 %, forum-only. | none | kept, flagged (revisit with a measured helm angle) |
| `heel_inertia` | 250 kg m^2 | No source. Mine: hiking sailor 80 kg at about 0.9 m, rig 22 kg at about 2 m, hull 58 kg at 0.4 m about the roll axis gives about 170 kg m^2; added inertia not computed | Order-of-magnitude check only. | low | kept, flagged |
| `heel_damping` | 900 N m s/rad | No source found. Searches found only generic roll-decay texts, nothing for a Laser. | n/a | none | kept, flagged |
| `yaw_inertia` | 200 kg m^2 | Forum rule of thumb only (boatdesign.net thread 329924: pitch gyradius about 25 % L, roll about 35 % B; no data for dinghies, and the contributor says he has none). Mine: 160 kg x (0.25 x 4.23 m)^2 = 180 kg m^2 before added inertia | Consistent with the rule of thumb. | low | kept, flagged |
| `yaw_damping_linear`, `yaw_damping_quadratic` | 80, 60 | No source found. Marimon Giovannetti et al. (2014, Procedia Eng. 72) and the Delft tacking papers (de Ridder, Vermeulen and Keuning 2004; Keuning et al. 2005) are the relevant model references but give the AC45 and yachts, and I could not read the Delft papers. | n/a | none | kept, flagged |
| `sway_mass` | 250 kg | No source. Mine, strip theory: hull sway added mass about rho pi Tc^2 Lwl = 1000 x pi x 0.094^2 x 3.81 = 106 kg (Tc from Day and Nixon 2014 Table 1, Bwl / Tc and Bwl); board adds roughly 20 to 30 kg (flat plate, chord 0.3 m, span 0.68 m, 3-D factor 0.6) | Total added mass of 100 to 140 kg on 160 kg; the sim has 90 kg. Same order. | low | kept, flagged |
| `surge_mass` | 170 kg | Day and Nixon (2014): 160 kg all-up (high). Added mass 10 kg | The 10 kg is my estimate (6 %); a slender hull's surge added mass is a few percent but I did not find a citation. | high for 160, none for the 10 | kept |
| `rudder_max_angle` | 35 deg | ILCA 2020 rules 15(d), (e): "maximum 78 degree rudder angle relative to the bottom edge of the rudder head". Lin et al. (2018): 470 CFD tested 0 to 40 deg, turning moment peaks at 35 deg | My reading of the ILCA 78 deg is the kick-up range of the blade in the head, not a steering throw; the rules give no tiller throw. Lin et al. support 35 deg only for a 470. | low | kept |
| `rudder_scull_moment`, `rudder_scull_speed` | 200 N m, 0.5 m/s | None | A modelling device for the effect of working the tiller at low speed (ticket 3, irons recovery). Not physical data. | none (model device) | kept |
| `low_speed_lateral_damping`, `low_speed_lateral_speed` | 400 N s/m, 0.5 m/s | None | A modelling device for the sideways resistance of the board when its foil lift vanishes at low speed. | none (model device) | kept |
| `rudder_arm` | 1.7 m | Day and Nixon (2014) Table 2: LCB at 0.532 Lwl from the forward perpendicular at 160 kg level trim; Lwl 3.81 m (Taylor 2016, Day and Nixon) | If the CG is near the LCB, the transom end of the waterline is 3.81 x (1 - 0.532) = 1.78 m aft of it; the rudder CP is at or slightly aft of the transom, so 1.8 to 1.9 m (mine). 1.7 m is within about 8 %. | medium-low (CG is assumed at LCB) | kept |
| `board_area` | 0.15 m^2 | ILCA rules measurement diagram (width 341 mm at mid-height, leading edge 680, thickness 33 mm) | No area given; the 2026-10-14 note estimates about 0.2 m^2 exposed. The comment says the value includes hull and rudder lateral area. | low | kept, flagged |
| `rudder_area` | 0.105 m^2 (RETUNED) | ILCA rules (blade width 203 mm); span 0.45 m is my estimate | Was 0.06, then 0.09 (= 0.45 m x 0.20 m, an estimate from the drawing), then 0.105 in Task 3 (the rudder stall and wake work). The 0.105 is a retune, not a source value: the 0.9 inflow factor cuts dynamic pressure to 0.81, and `tests/helm.rs` and `tests/steering.rs` then had no passing area below about 0.105 (measured grid in the code comment). The physical area is not known to better than 0.09 plus or minus 30 %. | none (retune against own tests) | kept, flagged honestly |
| `rudder_span` 0.45 m, `chord` 0.20 m (behind `aspect_ratio` 4.5) | AR_E = 2b/c = 4.5 | Chord: ILCA rules, blade width 203 mm (high). Span: my estimate, not published | The chord is sourced; the submerged span is not (it depends on blade rake and how far the blade is down). | high for chord, low for span | kept |
| rudder `efficiency` | 0.97 (was 0.9, assumed) | Day (2017) eq. 16 and 17; Nita and Scholz (2012) eq. 36 to 38 (read in the original, DLRK 2012, doc 281424) | See next section. | medium | changed to 0.97 |

## Rudder planform efficiency (Nita and Scholz 2012)

Source: Nita, M. and Scholz, D. (2012), "Estimating the Oswald Factor from Basic Aircraft Geometrical Parameters", Deutscher Luft- und Raumfahrtkongress 2012, document 281424, https://www.fzt.haw-hamburg.de/pers/Scholz/OPerA/OPerA_PUB_DLRK_12-09-10.pdf (read from the PDF, equations on p. 9, section 5).

The paper's theoretical Oswald factor (eq. 36 to 38):

- `f(l) = 0.0524 l^4 - 0.15 l^3 + 0.1659 l^2 - 0.0706 l + 0.0119`
- `dl = -0.357 + 0.45 exp(0.0375 sweep_25)`, sweep in degrees
- `e_theo = 1 / (1 + f(l - dl) A)`

Day (2017) eq. 17 reproduces exactly this (with TR for taper ratio and Lambda for sweep) and then uses `AR_E = AR e` in `CDi = CL^2 / (pi AR_E)` (eq. 16), with AR including the free-surface image. The paper then multiplies e_theo by correction factors for fuselage, zero-lift drag and Mach number (eq. 39a). Those are aircraft statistics and Day does not use them; neither does the sim. The paper itself calls e_theo "unrealistically high" for aircraft.

Applied to an untwisted rectangular to slightly tapered blade, sweep 0 (so dl = 0.093), AR as Day uses it (image included, 4.5; also 2.25 without the image):

| taper ratio | f(l - dl) | e at AR 4.5 | e at AR 2.25 |
|---|---|---|---|
| 0.8 | 0.00499 | 0.978 | 0.989 |
| 0.9 | 0.00636 | 0.972 | 0.986 |
| 1.0 | 0.00788 | 0.966 | 0.983 |

So e is 0.97 (0.966 to 0.978) and insensitive to the taper ratio. The sim's 0.9 was lower than this. Change: 0.9 to 0.97. Caveats: this is a theoretical, inviscid planform factor; the real blade has a raked leading edge, a gap at the hull and is near the free surface, which the image doubling only approximates. Confidence: medium (formula verified in the original, and matches Day's use; the blade taper is my assumption, but the result is flat in it).

Effect on the baseline (beam reach, fixed heading, hike 1, capsize disabled), before and after: 8 kn wind 4.62 / 4.62; 12 kn 6.51 / 6.51; 16 kn 8.88 / 8.88; 20 kn 11.71 / 11.71; 25 kn 15.03 / 15.04. The rudder carries little lift on a steady reach, so speeds are essentially unchanged (Day 2017: rudder induced drag is a few percent of total drag). Full `cargo test --workspace --no-fail-fast`: all tests pass, including `tests/helm.rs` and `tests/steering.rs`.

## Parameters considered for a change and not changed

- `hike_arm_extra` 0.15 to 0.03 on the Day 2017 estimate of about 750 N m: tried, `tests/hiking.rs` fails two tests, baseline speeds unchanged. Not committed (evidence is low, not medium, as explained in the table).
- Nothing else had a medium source disagreeing with the current value.

## Open questions

1. Boom height above the waterline and the depth of the board's centre of lateral resistance. The sail CE is 1.864 m above the boom, so `sail_heel_arm` 2.0 m looks low if the boom is 0.7 m or more above the water. Needs a Laser rig drawing (mast step, boom height at the gooseneck) or a photo scaled to the 5.12 m luff.
2. Mast position relative to the CG, to place `sail_ce_aft` (CE is 0.91 m aft of the luff). A hull and rig layout drawing would settle it.
3. What `hike` 0 and `hike` 1 mean physically. The sources (Day 2017, Taylor 2016) give only a maximum hiking moment, 713 to about 750 N m. The sim's 706 N m at `hike` 0 and 942 N m at `hike` 1 straddle it. Need a measured sitting-in moment to pin the split.
4. Boom angle close-hauled. Needs the traveller position and mainsheet block geometry from the ILCA measurement diagrams, or a measured sheet angle from Day (2017)'s trim data (the paper uses "rotation equal to sail trim angle" in the measured data; I did not find the numbers).
5. Added mass, roll and yaw inertia and damping for a dinghy. Delft (de Ridder, Vermeulen and Keuning 2004; Keuning et al. 2005; Verwerft and Keuning 2008) is likely to have yacht values, but the repository links returned HTML not PDFs and I could not read them.
6. Real rudder submerged span and the board's lateral area for the Laser, from the class foil templates or a measured blade.
7. Whether 35 deg is the real tiller-limited rudder throw; no ILCA figure found.
8. The scale of FLAT x SPILL x twist in Day's VPP at 12 to 16 kn (the values of f, t and s in the solved cases), which would let `depower_moment` and `depower_floor` be set from the VPP rather than a fit. Day's tables do not give them in the text.
