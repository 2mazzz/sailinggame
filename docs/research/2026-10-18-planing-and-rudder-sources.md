# Planing and rudder: sources and planing sensitivity (2026-10-18)

## Evidence table

Copied from `docs/plans/2026-10-18-planing-and-rudder-realism.md`.

Confidence is mine. "Fig." means read off a published figure.

| Number | Value | Source | Confidence |
|---|---|---|---|
| Hull drag area 2 to 9 kn | already in `params.rs` | Day and Nixon (2014) tank test, 160 kg, level trim | High (one hull) |
| Planing onset | about 6.5 kn, Froude number 0.55; sinkage turns negative and drag area starts to fall | Day and Nixon (2014) | High |
| Resistance above 9 kn | none | No source found | None |
| Generic planing threshold | Froude number about 1.0 to 1.2 on the length Froude number | Wikipedia "Hull speed" and planing-hull literature (secondary); the Laser is already past 0.55 at onset because the table is a transition regime | Low |
| Rudder lift slope | `5.7 AR_E / (1.8 + sqrt(AR_E² + 4))`, AR_E = 2b/c; gives 3.8 per rad for AR_E 4.5 | Day (2017) eq. 12, Keuning and Verwerft (2009) | Medium |
| Rudder induced drag | `CDi = CL² / (π AR_E)`, with AR_E = AR·e, e a planform efficiency (Nita and Scholz 2012) | Day (2017) eq. 16, 17 | Medium (e not yet read, assumed 0.9) |
| Rudder inflow speed | 0.9 × boat speed | Day (2017) | Medium |
| Rudder geometry | blade at most 20.3 mm thick, 203 mm wide; submerged span about 0.45 m (my estimate) | ILCA Class Rules 2020, measurement diagrams | High (outline), low (submerged span) |
| NACA 0012 stall | 14 degrees at Re 1.8e6, CLmax 1.33 smooth | NACA TN 3361 (Critzos 1955) | High for the section |
| Stall at dinghy Reynolds numbers | Re 0.36 to 0.7e6: CLmax about 1.0; a forum XFOIL run at Re 179,000 gives 9 to 11 degrees | Sandia SAND80-2114 (Sheldahl and Klimas 1981); forum post (low) | Medium |
| Dinghy rudder Reynolds number | about 3.5 to 5e5 (chord 0.20 m, 2 to 3 m/s) | My arithmetic | Medium |
| Rudder stall with hull | CFD 470 class, stall at 30 degrees rudder angle (35 at 10 m/s); best lift to drag at 10 to 25 degrees | Lin et al. 2018, Proceedings 2(6) 308 | Low: CFD, a 470 not a Laser, hull wake and board downwash included |
| Rudder share of drag | induced drag of the whole underwater body is 14 % of hydro drag upwind, under 1 % downwind | Day (2017) | Medium |

The 470 CFD figure (30 degrees) and the section figure (12 to 14 degrees) disagree. The section value applies to a blade in clean flow, which is what the sim models. The CFD value includes the hull's wake and downwash from the board, which cut the blade's effective angle. The plan keeps the section value and records the disagreement. Task 3 includes a decision point if testing shows it hurts steering.

## Planing tail sensitivity (measured)

Hull resistance above the 9 kn tank data is `R = R9 (v / 9 kn)^p` (`planing_resistance_exponent`). p = 2.0 reproduces the old constant drag area. Beam reach, fixed heading, sheet 0.55, hike 1, capsize disabled (`crates/sailsim/tests/baseline_planing_rudder.rs`). Speeds in knots.

| p | 8 kn wind | 12 kn | 16 kn | 20 kn | 25 kn |
|---|---|---|---|---|---|
| 0.0 | 4.62 | 6.51 | 8.88 | 12.22 | 16.00 |
| 0.5 (current) | 4.62 | 6.51 | 8.88 | 11.71 | 15.03 |
| 1.0 | 4.62 | 6.51 | 8.88 | 11.25 | 14.01 |
| 1.5 | 4.62 | 6.51 | 8.88 | 10.87 | 13.03 |
| 2.0 (old) | 4.62 | 6.51 | 8.88 | 10.57 | 12.19 |

Hull resistance at the same speeds, with W = 160 x 9.81 N:

| p | R(9 kn) | R(12 kn) | R(15 kn) | R/W at 9 | R/W at 12 | R/W at 15 |
|---|---|---|---|---|---|---|
| 0.0 | 221 N | 221 N | 221 N | 0.141 | 0.141 | 0.141 |
| 0.5 | 221 N | 255 N | 285 N | 0.141 | 0.162 | 0.182 |
| 1.0 | 221 N | 294 N | 368 N | 0.141 | 0.188 | 0.234 |
| 1.5 | 221 N | 340 N | 475 N | 0.141 | 0.217 | 0.303 |
| 2.0 | 221 N | 393 N | 613 N | 0.141 | 0.250 | 0.391 |

Speeds up to 16 kn wind are identical for every p because the boat stays under 9 kn.

## What is known and what is assumed

Known: the Day and Nixon (2014) tank test covers 2 to 9 kn for one Laser at 160 kg, level trim, and puts planing onset at about 6.5 kn. Assumed: everything above 9 kn. The power-law tail and its exponent p have no source; the table shows p moves the 25 kn reach speed between 12.2 and 16.0 kn, so the 25 kn figure is uncertain by about 4 kn from this choice alone. The test boat sails with hike = 1 and capsize disabled, so the simulated 25 kn speed is an upper bound for a real boat that must depower, and a real Laser would also be held back by waves and by capsize risk. We have no primary source for real Laser speeds on a reach in 25 kn of wind; the only figure is a forum anecdote of a GPS log of about 15 kn in a gust, with tide, which we cannot verify. The band test in `baseline_planing_rudder.rs` is a sanity band, not a validation.

## Sources

- Day, A.H. and Nixon, E. (2014). Measurement and prediction of the resistance of a Laser sailing dinghy. IJSCT. https://strathprints.strath.ac.uk/46407/
- Day, A.H. (2017). Performance prediction for sailing dinghies. Ocean Engineering. https://strathprints.strath.ac.uk/59980/
- Critzos, Heyson and Boswinkle (1955). NACA TN 3361. https://ntrs.nasa.gov/api/citations/19930084501/downloads/19930084501.pdf
- Sheldahl and Klimas (1981). SAND80-2114. https://digital.library.unt.edu/ark:/67531/metadc1191254/
- Lin, S. et al. (2018). Investigation on Rudder Hydrodynamics for 470 Class Yacht. Proceedings 2(6) 308. https://pdfs.semanticscholar.org/6fc7/c70307907c41e7cf68a455240f05abb2d9e8.pdf
- ILCA Class Rules 2020, World Sailing document 26057.
- Secondary or anecdotal, low confidence: Wikipedia "Hull speed"; boatdesign.net forum posts on NACA 0012 stall at Re 179,000; daysailer and sailing forum GPS reports for Laser speeds.
