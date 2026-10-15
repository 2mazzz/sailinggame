/// Metres per second in one knot.
const KN: f64 = 0.514_444;

/// Lift/drag characteristics of a foil (sail, centreboard, rudder).
#[derive(Debug, Clone, Copy)]
pub struct FoilParams {
    /// Attached-flow lift slope (per rad).
    pub lift_slope: f64,
    /// Angle of attack where stall blends in (rad).
    pub stall_angle: f64,
    pub drag_zero_lift: f64,
    pub aspect_ratio: f64,
    /// Flat-plate normal-force coefficient, used once stalled.
    pub plate_normal: f64,
    /// Exponent of the stall blend `x^n / (1 + x^n)`, `x = alpha / stall_angle`:
    /// higher keeps attached-flow lift further into the stall.
    pub stall_sharpness: i32,
    /// Soft ceiling on attached-flow lift (a trimmed sail tops out near 1.35,
    /// Day 2017); `None` leaves the lift slope unlimited.
    pub max_lift: Option<f64>,
}

/// Physical parameters of a boat, SI units.
#[derive(Debug, Clone, Copy)]
pub struct BoatParams {
    /// Hull + crew + added mass in surge (kg).
    pub surge_mass: f64,
    /// Bare-hull drag area A(u) = R / (0.5 rho u^2) as (speed m/s, m^2) points,
    /// linearly interpolated and held constant beyond the ends. It carries the
    /// hump near 6.5 kn where the hull starts to plane.
    pub hull_drag_area: [(f64, f64); 7],
    /// Sail area (m^2).
    pub sail_area: f64,
    /// Boom angle from the centreline when the sheet is fully eased / fully in (rad).
    /// The fully-in angle is a tuned value, no source (ticket 4): it keeps the no-go zone firm.
    pub boom_angle_eased: f64,
    pub boom_angle_in: f64,
    pub sail: FoilParams,
    /// Sail centre of effort, aft of the centre of gravity (m).
    pub sail_ce_aft: f64,
    /// Centreboard (centre of lateral resistance), forward of the centre of gravity (m).
    /// With the sail aft of the board the boat carries a little weather helm.
    pub board_arm: f64,
    /// Yaw inertia including added mass (kg m^2) and yaw damping
    /// (N m s/rad, N m s^2/rad^2).
    pub yaw_inertia: f64,
    pub yaw_damping_linear: f64,
    pub yaw_damping_quadratic: f64,
    pub rudder: FoilParams,
    pub rudder_area: f64,
    /// Distance from the centre of gravity to the rudder (m), and the rudder
    /// angle at full deflection (rad).
    pub rudder_arm: f64,
    pub rudder_max_angle: f64,
    /// Sculling moment (N m) at full rudder, peaking at about `rudder_scull_speed`
    /// (m/s) of surge: the extra turning a sailor gets by working the tiller
    /// when the blade alone is too slow to bite.
    pub rudder_scull_moment: f64,
    pub rudder_scull_speed: f64,
    /// Mass that the righting moment acts on: hull + crew (kg).
    pub displacement_mass: f64,
    /// Peak righting arm GZ (m), with the crew hiking.
    pub righting_arm_max: f64,
    /// Heel at which the righting moment vanishes (rad).
    pub vanishing_heel: f64,
    /// Heel at which the boat capsizes (mast in the water) (rad).
    pub capsize_heel: f64,
    /// Height of the sail's centre of effort above the board's centre of lateral resistance (m).
    pub sail_heel_arm: f64,
    /// Depowering (flattening the sail with outhaul, cunningham and vang):
    /// once the sail's heeling moment would pass `depower_moment` (N m) the
    /// sail force is eased toward `depower_floor` of its raw value, so heel
    /// and upwind speed level off in a breeze. Beyond the floor the boat can
    /// still be overpowered and capsize.
    pub depower_moment: f64,
    pub depower_floor: f64,
    /// Roll inertia (kg m^2) and roll damping (N m s/rad).
    pub heel_inertia: f64,
    pub heel_damping: f64,
    /// Hull + crew + added mass in sway (kg).
    pub sway_mass: f64,
    /// Centreboard (plus hull and rudder lateral area), area in m^2.
    pub board: FoilParams,
    pub board_area: f64,
    /// Low-speed sideways resistance (N s/m), fading out as surge speed
    /// approaches several times `low_speed_lateral_speed` (m/s).
    pub low_speed_lateral_damping: f64,
    pub low_speed_lateral_speed: f64,
}

impl BoatParams {
    /// A single-handed dinghy in the ILCA/Laser class: ~60 kg hull plus
    /// ~80 kg sailor, plus a little added mass.
    pub fn ilca_like() -> Self {
        Self {
            // Source: Day and Nixon (2014), 160 kg all-up (hull 58 kg + rig/foils ~22 kg + 80 kg sailor); plus ~10 kg added mass (my estimate). Confidence: high for 160, low for the added mass. See docs/research/2026-10-14-dinghy-physics-validation.md Q1.
            surge_mass: 170.0,
            // Source: Day and Nixon (2014) tank tests, bare Laser hull, 160 kg, level trim (Fig. 2 of Day 2017, read off). Speeds converted from knots. Confidence: high for the shape; one hull. See docs/research/2026-10-14-dinghy-physics-validation.md Q4.
            hull_drag_area: [
                (2.0 * KN, 0.0125),
                (3.0 * KN, 0.0132),
                (4.0 * KN, 0.0157),
                (5.0 * KN, 0.0207),
                (6.5 * KN, 0.0245),
                (8.0 * KN, 0.0221),
                (9.0 * KN, 0.0206),
            ],
            sail_area: 7.0,
            boom_angle_eased: 90.0_f64.to_radians(),
            boom_angle_in: 21.0_f64.to_radians(),
            sway_mass: 250.0,
            board: FoilParams {
                lift_slope: 3.1,
                stall_angle: 0.25,
                drag_zero_lift: 0.012,
                aspect_ratio: 2.0,
                plate_normal: 1.1,
                stall_sharpness: 2,
                max_lift: None,
            },
            board_area: 0.15,
            low_speed_lateral_damping: 400.0,
            low_speed_lateral_speed: 0.5,
            // Source: Day and Nixon (2014) benchmark all-up mass, hull plus rig plus foils plus crew (was 140 kg). Confidence: high.
            displacement_mass: 160.0,
            // Source: Taylor et al. (2016) measured 713 N m hiking moment for one subject; Day (2017) has a hiking crew's moment largest near upright. GZ 0.45 m on 160 kg gives about 700 N m. Confidence: medium (one subject). See docs/research/2026-10-14-dinghy-physics-validation.md Q6.
            righting_arm_max: 0.45,
            vanishing_heel: 85.0_f64.to_radians(),
            capsize_heel: 70.0_f64.to_radians(),
            sail_heel_arm: 2.0,
            // RETUNED, no source: set so upwind VMG levels off near 3.8 kn above 12 kn (Day 2017 Fig. 8) with about 2.5 deg of heel.
            depower_moment: 450.0,
            depower_floor: 0.35,
            heel_inertia: 250.0,
            heel_damping: 900.0,
            sail_ce_aft: 0.25,
            // RETUNED, no source (was 0.42): the larger rudder moved the centre of lateral resistance aft, giving lee helm; moving the board forward restores the mild weather helm (about 2 deg drift in 30 s) that tests/helm.rs expects. Research flags this CE/CLR lead as too large; revisit with the helm-angle data.
            board_arm: 0.76,
            yaw_inertia: 200.0,
            yaw_damping_linear: 80.0,
            yaw_damping_quadratic: 60.0,
            rudder: FoilParams {
                lift_slope: 3.5,
                stall_angle: 0.30,
                drag_zero_lift: 0.012,
                aspect_ratio: 2.5,
                plate_normal: 1.1,
                stall_sharpness: 2,
                max_lift: None,
            },
            // LOW confidence: my estimate (about 0.45 m x 0.2 m submerged) from the ILCA class rules drawing, not a published figure (was 0.06).
            rudder_area: 0.09,
            rudder_arm: 1.7,
            rudder_max_angle: 35.0_f64.to_radians(),
            rudder_scull_moment: 200.0,
            rudder_scull_speed: 0.5,
            sail: FoilParams {
                lift_slope: 6.2,
                stall_angle: 0.35,
                drag_zero_lift: 0.04,
                aspect_ratio: 4.5,
                plate_normal: 1.2,
                stall_sharpness: 4,
                max_lift: Some(1.15),
            },
        }
    }
}
