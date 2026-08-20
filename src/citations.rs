//! Citation registry: every literature claim the codebase makes, with its
//! verification status.
//!
//! Motivation: 36 source files carried `2026 SOTA` banners naming ~141 distinct
//! `Author Year` tokens, most without a DOI. Two were checked and found correct,
//! one was checked and found to carry a value the cited paper does not report,
//! and many could not be located in Crossref, OpenAlex or arXiv at all. A tool
//! whose selling point is honesty gating cannot cite sources it cannot produce.
//!
//! Rule enforced here: a citation may only be quoted as established if it is
//! present in [`VERIFIED`] with a resolvable identifier. Everything else is
//! either absent from the registry (and must not be quoted as fact) or listed in
//! [`UNVERIFIED`] with the reason.
//!
//! `scripts/audit_citations.py` scans the tree for author-year tokens and reports
//! any that are missing from this registry.

/// A citation whose existence and reported values were checked against a
/// bibliographic database.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Citation {
    /// Short token as it appears in source comments, e.g. "Umarhadi 2026".
    pub token: &'static str,
    pub authors: &'static str,
    pub year: u16,
    pub title: &'static str,
    pub venue: &'static str,
    /// DOI or arXiv id. Never empty for a verified entry.
    pub identifier: &'static str,
    /// What this codebase relies on from the paper, in its own reported terms.
    pub relied_upon: &'static str,
}

/// A citation the codebase referenced but which could not be confirmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnverifiedCitation {
    pub token: &'static str,
    /// Why it is not in [`VERIFIED`].
    pub reason: &'static str,
}

/// Citations confirmed against Crossref / OpenAlex / arXiv, with the reported
/// values this codebase quotes.
pub const VERIFIED: &[Citation] = &[
    Citation {
        token: "Hua 2026",
        authors: "Hua, Li, Li, Chai, Zhou & Wu",
        year: 2026,
        title: "Enhanced removal of anionic PFAS by electrically assisted nanofiltration",
        venue: "Journal of Hazardous Materials 504:141395",
        identifier: "10.1016/j.jhazmat.2026.141395",
        relied_upon: "PFOA/PFBS rejection and energy figures for the electro-NF design tool",
    },
    Citation {
        token: "Umarhadi 2026",
        authors: "Umarhadi & Siegert",
        year: 2026,
        title: "Monitoring degraded tropical peatland subsidence by integrating SBAS InSAR and machine learning",
        venue: "Environmental Research Communications 8:025025",
        identifier: "10.1088/2515-7620/ae43ab",
        relied_upon: "Mean peat subsidence -1.72 +/- 1.57 cm/yr (Block B), -1.55 +/- 2.27 cm/yr (Block C), ex-MRP Central Kalimantan",
    },
    Citation {
        token: "Aldiansyah 2024",
        authors: "Aldiansyah, Saputra, Wahid, Madani & Ningsih",
        year: 2024,
        title: "Rapid Flood Inundation Mapping Using Multi-Temporal Sentinel-1 SAR: An Example from Kendari City",
        venue: "Journal of Geospatial Remote Sensing (Unila)",
        identifier: "10.23960/jgrs.ft.unila.205",
        relied_upon: "Otsu thresholding on S1 in GEE: OA 95.81%, Kappa 0.86 for inundated area, Kendari",
    },
    Citation {
        token: "Venter 2022",
        authors: "Venter, Barton, Chakraborty, Simensen & Singh",
        year: 2022,
        title: "Global 10 m Land Use Land Cover Datasets: A Comparison of Dynamic World, World Cover and Esri Land Cover",
        venue: "Remote Sensing 14(16):4101",
        identifier: "10.3390/rs14164101",
        relied_upon: "Per-class accuracy 34-92%; overall DW 72%, Esri 75%, WorldCover 65%; recommends design-based inference over pixel counting",
    },
    Citation {
        token: "Brown 2022",
        authors: "Brown et al.",
        year: 2022,
        title: "Dynamic World, Near real-time global 10 m land use land cover mapping",
        venue: "Scientific Data 9",
        identifier: "10.1038/s41597-022-01307-4",
        relied_upon: "Dynamic World product definition and provenance",
    },
    Citation {
        token: "Olofsson 2014",
        authors: "Olofsson, Foody, Herold, Stehman, Woodcock & Wulder",
        year: 2014,
        title: "Good practices for estimating area and assessing accuracy of land change",
        venue: "Remote Sensing of Environment 148:42-57",
        identifier: "10.1016/j.rse.2014.02.015",
        relied_upon: "Unbiased area estimator (Eq. 4, 9, 10) — requires a real stratified reference sample",
    },
    Citation {
        token: "Susetyo 2023",
        authors: "Susetyo",
        year: 2023,
        title: "Vertical accuracy assessment of various open-source DEM data: DEMNAS, SRTM-1, and ASTER GDEM",
        venue: "Geodesy and Cartography 49(4):209-215",
        identifier: "10.3846/gac.2023.18168",
        relied_upon: "Indonesia vertical RMSE against GPS: SRTM-1 5.529 m, DEMNAS 8.172 m, ASTER GDEM 13.632 m",
    },
    Citation {
        token: "Hawker 2022",
        authors: "Hawker, Uhe, Paulo, Sosa, Savage, Sampson & Neal",
        year: 2022,
        title: "A 30 m global map of elevation with forests and buildings removed",
        venue: "Environmental Research Letters 17",
        identifier: "10.1088/1748-9326/ac4d4f",
        relied_upon: "FABDEM: MAE 1.61->1.12 m in built-up areas, 5.15->2.88 m in forest",
    },
    Citation {
        token: "Hawker 2024",
        authors: "Hawker, Neal, Savage, Kirkpatrick, Lord, Zylberberg, Groeger, Dang Thuy, Fox, Agyemang & Pham",
        year: 2024,
        title: "Assessing LISFLOOD-FP with the next-generation digital elevation model FABDEM",
        venue: "Natural Hazards and Earth System Sciences 24:539",
        identifier: "10.5194/nhess-24-539-2024",
        relied_upon: "FABDEM outperforms MERIT for flood modelling in the Central Highlands of Vietnam (tropical SE Asia)",
    },
    Citation {
        token: "Sahid 2024",
        authors: "Sahid",
        year: 2024,
        title: "Enhancing Digital Elevation Model Accuracy for Flood Modelling - A Case Study of the Ciberes River in Cirebon Indonesia",
        venue: "Forum Geografi 38(1)",
        identifier: "10.23917/forgeo.v38i1.1839",
        relied_upon: "Flood-depth accuracy +11.67% from DEM filtering, +24.98% with measured river cross-section added",
    },
    Citation {
        token: "Vollrath 2020",
        authors: "Vollrath, Mullissa & Reiche",
        year: 2020,
        title: "Angular-Based Radiometric Slope Correction for Sentinel-1 on Google Earth Engine",
        venue: "Remote Sensing 12(11):1867",
        identifier: "10.3390/rs12111867",
        relied_upon: "Slope correction with layover/shadow masking, CEOS-compliant, open GEE module",
    },
    Citation {
        token: "Truckenbrodt 2019",
        authors: "Truckenbrodt et al.",
        year: 2019,
        title: "Towards Sentinel-1 SAR Analysis-Ready Data: A Best Practices Assessment",
        venue: "Data 4(3):93",
        identifier: "10.3390/data4030093",
        relied_upon: "Radiometric terrain correction best practice; tested over Fiji (tropical) and the Alps",
    },
    Citation {
        token: "Bereczky 2022",
        authors: "Bereczky, Wieland, Krullikowski, Martinis & Plank",
        year: 2022,
        title: "Sentinel-1-Based Water and Flood Mapping: Benchmarking CNNs Against an Operational Rule-Based Processing Chain",
        venue: "IEEE JSTARS",
        identifier: "10.1109/jstars.2022.3152127",
        relied_upon: "Dual-pol beats single-pol by 5% IoU; radiometric augmentation helps, geometric degrades",
    },
    Citation {
        token: "Bonafilia 2020",
        authors: "Bonafilia, Tellman, Anderson & Issenberg",
        year: 2020,
        title: "Sen1Floods11: a georeferenced dataset to train and test deep learning flood algorithms for Sentinel-1",
        venue: "CVPRW 2020",
        identifier: "10.1109/cvprw50498.2020.00113",
        relied_upon: "Reference flood benchmark dataset",
    },
    Citation {
        token: "Bai 2021",
        authors: "Bai, Wu, Yang, Yu, Zhao, Liu, Yang, Mas & Koshimura",
        year: 2021,
        title: "Enhancement of Detecting Permanent Water and Temporary Water in Flood Disasters by Fusing Sentinel-1 and Sentinel-2",
        venue: "Remote Sensing 13(11):2220",
        identifier: "10.3390/rs13112220",
        relied_upon: "Sen1Floods11 S1+S2 fusion: mIoU 52.99%, IoU 52.30%, OA 92.81%",
    },
    Citation {
        token: "Amitrano 2024",
        authors: "Amitrano, Di Martino, Di Simone & Imperatore",
        year: 2024,
        title: "Flood Detection with SAR: A Review of Techniques and Datasets",
        venue: "Remote Sensing 16(4):656",
        identifier: "10.3390/rs16040656",
        relied_upon: "SAR flood mapping remains severely limited in vegetated and urban areas",
    },
    Citation {
        token: "Maslukah 2026",
        authors: "Maslukah",
        year: 2026,
        title: "Evaluation and Local Calibration of Sentinel-2 Chlorophyll-a Algorithms in Kendal Coastal Waters, Indonesia",
        venue: "International Journal of Geoinformatics 22(4)",
        identifier: "10.52939/ijg.v22i4.4937",
        relied_upon: "Locally calibrated single-band (green) beats imported algorithms: RMSE 0.74 vs 0.89 vs 0.93 ug/L",
    },
    Citation {
        token: "Mishra 2012",
        authors: "Mishra & Mishra",
        year: 2012,
        title: "Normalized difference chlorophyll index: A novel model for remote estimation of chlorophyll-a",
        venue: "Remote Sensing of Environment 117:394-406",
        identifier: "10.1016/j.rse.2011.10.016",
        relied_upon: "NDCI formulation used by the water-quality engine",
    },
    Citation {
        token: "Dogliotti 2015",
        authors: "Dogliotti, Ruddick, Nechad, Doxaran & Knaeps",
        year: 2015,
        title: "A single algorithm to retrieve turbidity from remotely-sensed data in all coastal and estuarine waters",
        venue: "Remote Sensing of Environment 156:157-168",
        identifier: "10.1016/j.rse.2014.09.020",
        relied_upon: "Two-branch turbidity retrieval with 7-15 FNU transition weighting",
    },
    Citation {
        token: "Klemes 1986",
        authors: "Klemes",
        year: 1986,
        title: "Operational testing of hydrological simulation models",
        venue: "Hydrological Sciences Journal 31(1):13-24",
        identifier: "10.1080/02626668609491024",
        relied_upon: "Split-sample test: contiguous, not random, splitting of autocorrelated series",
    },
    Citation {
        token: "Moriasi 2007",
        authors: "Moriasi, Arnold, Van Liew, Bingner, Harmel & Veith",
        year: 2007,
        title: "Model evaluation guidelines for systematic quantification of accuracy in watershed simulations",
        venue: "Transactions of the ASABE 50(3):885-900",
        identifier: "10.13031/2013.23153",
        relied_upon: "Satisfactory thresholds NSE >= 0.50, |PBIAS| <= 25% for streamflow",
    },
    Citation {
        token: "Beven 1992",
        authors: "Beven & Binley",
        year: 1992,
        title: "The future of distributed models: model calibration and uncertainty prediction",
        venue: "Hydrological Processes 6(3):279-298",
        identifier: "10.1002/hyp.3360060305",
        relied_upon: "GLUE informal Bayesian uncertainty estimation",
    },
    Citation {
        token: "Evensen 1994",
        authors: "Evensen",
        year: 1994,
        title: "Sequential data assimilation with a nonlinear quasi-geostrophic model using Monte Carlo methods",
        venue: "Journal of Geophysical Research 99(C5):10143-10162",
        identifier: "10.1029/94JC00572",
        relied_upon: "Ensemble Kalman Filter formulation",
    },
];

/// Tokens the codebase referenced that could not be confirmed in Crossref,
/// OpenAlex or arXiv. Absence of a search result is not proof a paper does not
/// exist, but it is sufficient reason not to present it as established.
pub const UNVERIFIED: &[UnverifiedCitation] = &[
    UnverifiedCitation {
        token: "Sun 2026",
        reason: "cited in enkf.rs as 'IoU water quality ADAPT, beats EnKF'; not found in Crossref/OpenAlex/arXiv; no DOI given",
    },
    UnverifiedCitation {
        token: "Sahar 2026",
        reason: "cited in enkf.rs as 'fault detection EnKF'; not located; no DOI given",
    },
    UnverifiedCitation {
        token: "Sandu 2026",
        reason: "cited in enkf.rs as 'atmospheric composition DA'; not located; no DOI given",
    },
    UnverifiedCitation {
        token: "Hammoud 2026",
        reason: "cited in enkf.rs as 'RL+Bayesian'; not located; no DOI given",
    },
    UnverifiedCitation {
        token: "Shirkhani 2026",
        reason: "was cited on the mineral prospectivity figure as the source of XGBoost/SHAP weights; not located. The method is a weighted linear sum with expert weights, so the attribution was removed rather than repaired",
    },
    UnverifiedCitation {
        token: "Ostrowski 2026",
        reason: "cited in foundation_models.rs as EGU 2026; not located. Module now returns not_implemented",
    },
    UnverifiedCitation {
        token: "Kacmaz 2026",
        reason: "cited in flood_sar.rs as 'Siamese U-Net, F1=96%'; not located. Figure exceeds every verified SAR flood benchmark",
    },
    UnverifiedCitation {
        token: "Ahmadi 2026",
        reason: "cited in flood_sar.rs as 'TLE-FEDformer, 98.1%'; not located. Figure exceeds every verified SAR flood benchmark",
    },
    UnverifiedCitation {
        token: "Kinalioglu 2026",
        reason: "cited in flood_sar.rs as 'LightFloodNet, 1.57M params'; not located",
    },
    UnverifiedCitation {
        token: "Gierszewska 2026",
        reason: "cited in flood_sar.rs as 'RS-Mamba'; not located",
    },
    UnverifiedCitation {
        token: "Widiarso 2026",
        reason: "cited in mintpy_insar.rs for Semarang subsidence; not located. Attribution removed; Semarang value now sourced to Science Advances 2024",
    },
    UnverifiedCitation {
        token: "Setyaningrum 2026",
        reason: "cited in mintpy_insar.rs for Central Kalimantan; not located. Attribution removed; peat values now sourced to Umarhadi & Siegert 2026",
    },
    UnverifiedCitation {
        token: "Pratama 2026",
        reason: "cited in mintpy_insar.rs for Jatiluhur Dam; not located. Entry removed",
    },
];

/// Look up a verified citation by its short token.
pub fn verified(token: &str) -> Option<&'static Citation> {
    VERIFIED.iter().find(|c| c.token.eq_ignore_ascii_case(token))
}

/// Whether a token is explicitly recorded as unverified.
pub fn is_unverified(token: &str) -> Option<&'static UnverifiedCitation> {
    UNVERIFIED
        .iter()
        .find(|c| c.token.eq_ignore_ascii_case(token))
}

/// Render a citation for inclusion in tool output, with its identifier.
pub fn cite(token: &str) -> String {
    match verified(token) {
        Some(c) => format!(
            "{} ({}), {}, {} [{}]",
            c.authors, c.year, c.title, c.venue, c.identifier
        ),
        None => match is_unverified(token) {
            Some(u) => format!(
                "[SITASI TIDAK TERVERIFIKASI: {}] {} — tidak boleh dikutip sebagai fakta",
                u.token, u.reason
            ),
            None => format!(
                "[SITASI TIDAK TERDAFTAR: {}] tidak ada di registry src/citations.rs",
                token
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_verified_citation_carries_an_identifier() {
        for c in VERIFIED {
            assert!(
                !c.identifier.trim().is_empty(),
                "verified citation '{}' has no DOI/arXiv id",
                c.token
            );
            assert!(
                c.identifier.starts_with("10.") || c.identifier.starts_with("arXiv"),
                "identifier for '{}' is not a DOI or arXiv id: {}",
                c.token,
                c.identifier
            );
        }
    }

    #[test]
    fn every_verified_citation_states_what_is_relied_upon() {
        for c in VERIFIED {
            assert!(
                !c.relied_upon.trim().is_empty(),
                "verified citation '{}' does not say what is relied upon",
                c.token
            );
        }
    }

    #[test]
    fn no_token_is_both_verified_and_unverified() {
        for u in UNVERIFIED {
            assert!(
                verified(u.token).is_none(),
                "'{}' appears in both VERIFIED and UNVERIFIED",
                u.token
            );
        }
    }

    #[test]
    fn tokens_are_unique() {
        let mut seen: Vec<&str> = VERIFIED.iter().map(|c| c.token).collect();
        let before = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(before, seen.len(), "duplicate token in VERIFIED");
    }

    #[test]
    fn unverified_entries_explain_themselves() {
        for u in UNVERIFIED {
            assert!(
                u.reason.len() > 20,
                "'{}' needs a substantive reason, got: {}",
                u.token,
                u.reason
            );
        }
    }

    #[test]
    fn cite_renders_verified_with_doi() {
        let s = cite("Umarhadi 2026");
        assert!(s.contains("10.1088/2515-7620/ae43ab"), "got: {s}");
    }

    #[test]
    fn cite_flags_unverified_and_unregistered() {
        assert!(cite("Kacmaz 2026").contains("TIDAK TERVERIFIKASI"));
        assert!(cite("Nobody 1999").contains("TIDAK TERDAFTAR"));
    }

    #[test]
    fn peat_subsidence_value_matches_the_cited_paper() {
        // Guards the specific error this registry was built to catch: the
        // Kalimantan rate must not drift back to -50 mm/yr.
        let c = verified("Umarhadi 2026").expect("registered");
        assert!(
            c.relied_upon.contains("-1.72") && c.relied_upon.contains("-1.55"),
            "reported peat rates must stay as published: {}",
            c.relied_upon
        );
        assert!(!c.relied_upon.contains("-50"));
    }
}
