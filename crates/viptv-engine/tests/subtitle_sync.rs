use viptv_engine::subtitle_sync::*;

fn cues(duration_s: i64, seed: u64) -> Vec<CueInterval> {
    let mut state = seed;
    let mut result = Vec::new();
    let mut t = 0;
    while t < duration_s * 1_000_000 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        t += (3 + ((state >> 32) % 30) as i64) * 100_000;
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let end = t + (4 + ((state >> 32) % 35) as i64) * 100_000;
        result.push(CueInterval {
            start_us: t,
            end_us: end,
        });
        t = end;
    }
    result
}

fn observations(
    cues: &[CueInterval],
    map: Correction,
    start_s: i64,
    seconds: i64,
) -> Vec<Observation> {
    (start_s * 50..(start_s + seconds) * 50)
        .map(|frame| {
            let start_us = frame * 20_000;
            let speech = cues.iter().any(|c| {
                let start = map.title_time_us(c.start_us).unwrap();
                let end = map.title_time_us(c.end_us).unwrap();
                start_us + 10_000 >= start && start_us + 10_000 < end
            });
            Observation {
                start_us,
                end_us: start_us + 20_000,
                activity: if speech {
                    Activity::Speech
                } else {
                    Activity::Silence
                },
            }
        })
        .collect()
}

#[test]
fn positive_and_negative_offset_use_title_time_sign() {
    for offset_us in [-7_300_000, 2_400_000, 0] {
        let cues = cues(180, 42);
        let mut estimator = Estimator::new(7, cues.clone()).unwrap();
        let truth = Correction {
            anchor_us: 0,
            offset_us,
            scale: 1.0,
        };
        estimator
            .push(7, &observations(&cues, truth, 40, 60))
            .unwrap();
        let estimate = estimator.estimate(7).unwrap();
        assert_eq!(estimate.state, State::Accepted, "{estimate:?}");
        assert!((estimate.correction.unwrap().offset_us - offset_us).abs() <= 100_000);
        assert_eq!(estimate.revision, 1);
        assert_eq!(estimate.algorithm_version, ALGORITHM_VERSION);
    }
}

#[test]
fn explicit_and_implicit_gaps_are_unobserved_not_silence() {
    let track = cues(180, 42);
    for explicit in [false, true] {
        let mut estimator = Estimator::new(1, track.clone()).unwrap();
        let mut samples = observations(
            &track,
            Correction {
                anchor_us: 0,
                offset_us: 0,
                scale: 1.0,
            },
            40,
            60,
        );
        samples.retain_mut(|o| {
            if o.start_us >= 50_000_000 && o.start_us < 80_000_000 {
                o.activity = Activity::Gap;
                explicit
            } else {
                true
            }
        });
        estimator.push(1, &samples).unwrap();
        let result = estimator.estimate(1).unwrap();
        assert_eq!(result.state, State::Unavailable);
        assert_eq!(result.evidence.observed_us, 30_000_000);
        assert_eq!(result.correction, None);
    }
}

#[test]
fn silence_sparse_sustained_activity_and_mismatched_activity_abstain() {
    let track = cues(180, 42);
    let other = cues(180, 754);
    let mut mismatched = Estimator::new(1, track).unwrap();
    mismatched
        .push(
            1,
            &observations(
                &other,
                Correction {
                    anchor_us: 0,
                    offset_us: 0,
                    scale: 1.0,
                },
                40,
                60,
            ),
        )
        .unwrap();
    assert_ne!(mismatched.estimate(1).unwrap().state, State::Accepted);
    for activity in [Activity::Silence, Activity::Speech] {
        let mut estimator = Estimator::new(1, cues(180, 42)).unwrap();
        estimator
            .push(
                1,
                &[Observation {
                    start_us: 40_000_000,
                    end_us: 100_000_000,
                    activity,
                }],
            )
            .unwrap();
        assert_eq!(estimator.estimate(1).unwrap().state, State::Unavailable);
    }
    let sparse = vec![CueInterval {
        start_us: 45_000_000,
        end_us: 46_000_000,
    }];
    let mut estimator = Estimator::new(1, sparse.clone()).unwrap();
    estimator
        .push(
            1,
            &observations(
                &sparse,
                Correction {
                    anchor_us: 0,
                    offset_us: 0,
                    scale: 1.0,
                },
                40,
                60,
            ),
        )
        .unwrap();
    assert_ne!(estimator.estimate(1).unwrap().state, State::Accepted);
}

#[test]
fn repetitive_activity_has_multiple_peaks_and_abstains() {
    let track: Vec<_> = (0..100)
        .map(|i| CueInterval {
            start_us: i * 2_000_000,
            end_us: i * 2_000_000 + 1_000_000,
        })
        .collect();
    let mut estimator = Estimator::new(1, track.clone()).unwrap();
    estimator
        .push(
            1,
            &observations(
                &track,
                Correction {
                    anchor_us: 0,
                    offset_us: 0,
                    scale: 1.0,
                },
                40,
                60,
            ),
        )
        .unwrap();
    let result = estimator.estimate(1).unwrap();
    assert_eq!(result.state, State::Ambiguous);
    assert_eq!(result.correction, None);
}

#[test]
fn drift_requires_ten_minutes_of_independent_anchors() {
    let track = cues(1_200, 42);
    let truth = Correction {
        anchor_us: 0,
        offset_us: 1_200_000,
        scale: 1.001,
    };
    let mut estimator = Estimator::new(9, track.clone()).unwrap();
    for window in 0..13 {
        let start = 40 + window * 60;
        estimator
            .push(9, &observations(&track, truth, start, 60))
            .unwrap();
        let result = estimator.estimate(9).unwrap();
        let correction = result.correction.unwrap();
        if window < 11 {
            assert_eq!(correction.scale, 1.0, "{window}: {result:?}");
        }
        if window == 12 {
            assert!(
                (correction.scale - truth.scale).abs() < 0.0002,
                "{result:?}"
            );
            let target = 1_000_000_000;
            assert!(
                (correction.title_time_us(target).unwrap() - truth.title_time_us(target).unwrap())
                    .abs()
                    <= 200_000
            );
        }
    }
}

#[test]
fn epochs_invalid_batches_and_repeat_estimates_do_not_mutate_state() {
    let mut estimator = Estimator::new(3, cues(180, 42)).unwrap();
    let sample = Observation {
        start_us: 0,
        end_us: 100_000,
        activity: Activity::Speech,
    };
    assert_eq!(estimator.push(2, &[sample]), Err(SyncError::StaleEpoch));
    assert_eq!(estimator.estimate(2).unwrap_err(), SyncError::StaleEpoch);
    assert_eq!(
        estimator.push(3, &[sample, sample]),
        Err(SyncError::InvalidInput)
    );
    estimator.push(3, &[sample]).unwrap();
    assert_eq!(estimator.estimate(3).unwrap().revision, 0);
    assert_eq!(
        estimator.push(3, &vec![sample; MAX_BATCH + 1]),
        Err(SyncError::InvalidInput)
    );
    assert!(
        Estimator::new(
            0,
            vec![CueInterval {
                start_us: 2,
                end_us: 1
            }]
        )
        .is_err()
    );
}

#[test]
fn large_timestamps_mapping_origin_manual_delay_and_inverse() {
    let map = Correction {
        anchor_us: 600_000_000,
        offset_us: 2_000_000,
        scale: 1.001,
    };
    assert_eq!(
        map.player_time_us(650_000_000, 60_000_000, 0),
        Some(592_050_000)
    );
    assert_eq!(map.mpv_delay_us(60_000_000, 0), Some(1_460_000));
    assert_eq!(
        map.subtitle_time_us(652_550_000, 500_000),
        Some(650_000_000)
    );
    let base = 8_000_000_000_000_000_000_i64;
    let unit = Correction {
        anchor_us: 0,
        offset_us: 17,
        scale: 1.0,
    };
    assert_eq!(unit.title_time_us(base + 3), Some(base + 20));
    assert_eq!(unit.subtitle_time_us(base + 23, 3), Some(base + 3));
    let large = Correction {
        anchor_us: base,
        offset_us: 2_000_000,
        scale: 1.001,
    };
    assert_eq!(
        large.title_time_us(base + 50_000_000),
        Some(base + 52_050_000)
    );
    assert_eq!(
        large.subtitle_time_us(base + 52_050_000, 0),
        Some(base + 50_000_000)
    );
    let track: Vec<_> = cues(180, 42)
        .into_iter()
        .map(|c| CueInterval {
            start_us: c.start_us + base,
            end_us: c.end_us + base,
        })
        .collect();
    let truth = Correction {
        anchor_us: base,
        offset_us: 2_400_000,
        scale: 1.0,
    };
    let mut estimator = Estimator::new(1, track.clone()).unwrap();
    estimator
        .push(1, &observations(&track, truth, base / 1_000_000 + 40, 60))
        .unwrap();
    assert_eq!(
        estimator.estimate(1).unwrap().correction.unwrap().offset_us,
        2_400_000
    );
}

#[test]
fn out_of_search_range_and_different_edit_abstain() {
    let track = cues(180, 42);
    for offset_us in [45_000_000, -45_000_000, 30_200_000, -30_200_000] {
        let mut estimator = Estimator::new(1, track.clone()).unwrap();
        estimator
            .push(
                1,
                &observations(
                    &track,
                    Correction {
                        anchor_us: 0,
                        offset_us,
                        scale: 1.0,
                    },
                    40,
                    60,
                ),
            )
            .unwrap();
        assert_ne!(estimator.estimate(1).unwrap().state, State::Accepted);
    }
    let mut estimator = Estimator::new(1, track.clone()).unwrap();
    let mut edited = observations(
        &track,
        Correction {
            anchor_us: 0,
            offset_us: 0,
            scale: 1.0,
        },
        40,
        30,
    );
    edited.extend(observations(
        &track,
        Correction {
            anchor_us: 0,
            offset_us: 5_000_000,
            scale: 1.0,
        },
        70,
        30,
    ));
    estimator.push(1, &edited).unwrap();
    assert_ne!(estimator.estimate(1).unwrap().state, State::Accepted);
}

#[test]
fn correction_evidence_is_stable_when_new_evidence_abstains() {
    let track = cues(300, 42);
    let mut estimator = Estimator::new(1, track.clone()).unwrap();
    estimator
        .push(
            1,
            &observations(
                &track,
                Correction {
                    anchor_us: 0,
                    offset_us: 2_000_000,
                    scale: 1.0,
                },
                40,
                60,
            ),
        )
        .unwrap();
    let accepted = estimator.estimate(1).unwrap();
    estimator
        .push(
            1,
            &[Observation {
                start_us: 100_000_000,
                end_us: 160_000_000,
                activity: Activity::Gap,
            }],
        )
        .unwrap();
    let absent = estimator.estimate(1).unwrap();
    assert_eq!(absent.state, State::Unavailable);
    assert_eq!(absent.correction, accepted.correction);
    assert_eq!(absent.revision, accepted.revision);
    assert!(absent.observation_revision > accepted.observation_revision);
    assert_eq!(
        absent.correction_evidence.unwrap().end_us,
        accepted.correction_evidence.unwrap().end_us
    );
    assert_eq!(absent.evidence.observed_us, 0);
}

#[test]
fn partial_frames_missing_subframes_and_large_gaps_are_bounded() {
    let mut estimator = Estimator::new(1, cues(180, 42)).unwrap();
    estimator
        .push(
            1,
            &[Observation {
                start_us: 0,
                end_us: 59_980_000,
                activity: Activity::Speech,
            }],
        )
        .unwrap();
    assert_eq!(estimator.estimate(1).unwrap().state, State::Observing);
    estimator
        .push(
            1,
            &[Observation {
                start_us: 60_000_000,
                end_us: 60_020_000,
                activity: Activity::Silence,
            }],
        )
        .unwrap();
    assert_eq!(
        estimator.estimate(1).unwrap().evidence.observed_us,
        59_900_000
    );
    estimator
        .push(
            1,
            &[Observation {
                start_us: 4_000_000_000_000,
                end_us: 4_000_060_000_000,
                activity: Activity::Gap,
            }],
        )
        .unwrap();
    assert_eq!(estimator.estimate(1).unwrap().evidence.observed_us, 0);
    assert_eq!(
        estimator.push(
            1,
            &[Observation {
                start_us: 4_000_060_000_000,
                end_us: 4_000_241_000_000,
                activity: Activity::Silence
            }]
        ),
        Err(SyncError::InvalidInput)
    );
}

#[test]
fn unsupported_offset_jump_does_not_relabel_previous_correction_as_current() {
    let track = cues(300, 42);
    let mut estimator = Estimator::new(1, track.clone()).unwrap();
    estimator
        .push(
            1,
            &observations(
                &track,
                Correction {
                    anchor_us: 0,
                    offset_us: 0,
                    scale: 1.0,
                },
                40,
                60,
            ),
        )
        .unwrap();
    let first = estimator.estimate(1).unwrap();
    estimator
        .push(
            1,
            &observations(
                &track,
                Correction {
                    anchor_us: 0,
                    offset_us: 5_000_000,
                    scale: 1.0,
                },
                100,
                60,
            ),
        )
        .unwrap();
    let second = estimator.estimate(1).unwrap();
    assert_eq!(second.state, State::Observing);
    assert_eq!(second.correction, first.correction);
    assert_eq!(second.revision, first.revision);
}

#[test]
fn negative_drift_is_fitted_and_nonlinear_edit_is_not() {
    let track = cues(1_200, 42);
    let truth = Correction {
        anchor_us: 0,
        offset_us: -1_200_000,
        scale: 0.999,
    };
    let mut drifting = Estimator::new(1, track.clone()).unwrap();
    let mut edited = Estimator::new(2, track.clone()).unwrap();
    for window in 0..13 {
        drifting
            .push(1, &observations(&track, truth, 40 + window * 60, 60))
            .unwrap();
        drifting.estimate(1).unwrap();
        edited
            .push(
                2,
                &observations(
                    &track,
                    Correction {
                        anchor_us: 0,
                        offset_us: (window % 2) * 4_000_000,
                        scale: 1.0,
                    },
                    40 + window * 60,
                    60,
                ),
            )
            .unwrap();
        edited.estimate(2).unwrap();
    }
    let result = drifting.estimate(1).unwrap();
    assert!(
        (result.correction.unwrap().scale - truth.scale).abs() < 0.0002,
        "{result:?}"
    );
    let evidence = result.correction_evidence.unwrap();
    assert!(evidence.anchor_span_us >= 600_000_000);
    assert!(evidence.scale_uncertainty_ppm.unwrap() <= 500.0);
    let result = edited.estimate(2).unwrap();
    assert_eq!(result.correction.unwrap().scale, 1.0);
    assert_eq!(result.revision, 1);
}

#[test]
fn old_anchors_expire_across_large_observed_gap() {
    let first = cues(180, 42);
    let base = 4_000_000_000_000;
    let later: Vec<_> = first
        .iter()
        .map(|c| CueInterval {
            start_us: base + c.start_us,
            end_us: base + c.end_us,
        })
        .collect();
    let mut track = first.clone();
    track.extend(later.iter().copied());
    let map = Correction {
        anchor_us: 0,
        offset_us: 0,
        scale: 1.0,
    };
    let mut estimator = Estimator::new(1, track).unwrap();
    estimator
        .push(1, &observations(&first, map, 40, 60))
        .unwrap();
    let accepted = estimator.estimate(1).unwrap();
    estimator
        .push(
            1,
            &[Observation {
                start_us: 100_000_000,
                end_us: base,
                activity: Activity::Gap,
            }],
        )
        .unwrap();
    assert_eq!(estimator.estimate(1).unwrap().evidence.anchors, 0);
    estimator
        .push(1, &observations(&later, map, base / 1_000_000 + 40, 60))
        .unwrap();
    let result = estimator.estimate(1).unwrap();
    assert_eq!(result.evidence.anchors, 1);
    assert_eq!(result.revision, accepted.revision);
}
