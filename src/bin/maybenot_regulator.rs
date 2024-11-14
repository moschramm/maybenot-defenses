// Maybenot RegulaTor -- uses constant-rate traffic to approximate the RegulaTor defense
// Code from the paper "State Machine Frameworks for Website Fingerprinting Defenses: Maybe Not"

use enum_map::enum_map;
use std::env;
use std::f64::INFINITY;

use maybenot::{
    action::Action,
    constants::STATE_END,
    dist::{Dist, DistType},
    event::Event,
    state::State,
    state::Trans,
    Machine,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(
        args.len() == 6,
        "Usage: {} <initial rate> <decay rate> <threshold> <upload ratio> <cells per state>",
        &args[0]
    );

    let initial_rate: f64 = args[1].parse().expect("Invalid initial rate"); // RegulaTor param = R, initial surge rate (packets / sec)
    let decay_rate: f64 = args[2].parse().expect("Invalid decay rate"); // RegulaTor param = D, decay rate
    let threshold: f64 = args[3].parse().expect("Invalid threshold"); // RegulaTor param = T, surge threshold
    let upload_ratio: f64 = args[4].parse().expect("Invalid upload ratio"); // RegulaTor param = U, upload ratio
    let packets_per_state: f64 = args[5].parse().expect("Invalid packets per state"); // number of packets per state (approximation granularity)

    let relay_machine =
        generate_relay_machine(packets_per_state, initial_rate, decay_rate, threshold);
    println!("Relay machine: {} ({})", relay_machine, relay_machine.len());
    println!();

    let client_machine = generate_client_machine(upload_ratio);
    println!(
        "Client machine: {} ({})",
        client_machine,
        client_machine.len()
    );
    println!();
}

// Generate a RegulaTor client-side machine.
fn generate_client_machine(upload_ratio: f64) -> String {
    // Set up state vector
    let num_states = (upload_ratio as usize) + 1;
    let prob_last_trans = 1.0 - upload_ratio.fract() as f32;

    let mut states: Vec<State> = Vec::with_capacity(num_states);

    // COUNTER states
    for i in 1..num_states {
        let mut prob_trans = 1.0;
        if i == num_states - 1 {
            prob_trans = prob_last_trans;
        }

        states.push(generate_client_count_state(i - 1, i, prob_trans));
    }

    // SEND state
    states.push(generate_client_send_state());

    // Machine construction
    let machine = Machine {
        allowed_padding_packets: u64::MAX,
        max_padding_frac: 0.0,
        allowed_blocked_microsec: u64::MAX,
        max_blocking_frac: 0.0,
        states: states,
    };

    return machine.serialize();
}

fn generate_client_send_state() -> State {
    // SEND state
    let mut state = State::new(enum_map! {
        Event::PaddingSent => vec![Trans(0, 1.0)],
        _ => vec![],
    });

    let timeout = Dist::new(
        DistType::Uniform {
            low: 0.0,
            high: 0.0,
        },
        0.0,
        0.0,
    );

    state.action = Some(Action::SendPadding {
        bypass: true,
        replace: true,
        timeout: timeout,
        limit: None,
    });

    return state;
}

fn generate_client_count_state(curr_index: usize, next_index: usize, prob_trans: f32) -> State {
    // Transition to next COUNTER state or stay in current one
    let mut state;

    if prob_trans < 1.0 {
        state = State::new(enum_map! {
            Event::PaddingRecv => vec![Trans(next_index, prob_trans), Trans(curr_index, 1.0 - prob_trans)],
            Event::NormalRecv => vec![Trans(next_index, prob_trans), Trans(curr_index, 1.0 - prob_trans)],
            Event::LimitReached => vec![Trans(next_index, 1.0)],
            _ => vec![],
        });
    } else {
        state = State::new(enum_map! {
            Event::PaddingRecv => vec![Trans(next_index, prob_trans)],
            Event::NormalRecv => vec![Trans(next_index, prob_trans)],
            _ => vec![],
        });
    }

    let timeout = Dist::new(
        DistType::Uniform {
            low: 0.0,
            high: 0.0,
        },
        0.0,
        0.0,
    );

    let limit = Dist::new(
        DistType::Uniform {
            low: 2.0,
            high: 2.0,
        },
        0.0,
        0.0,
    );

    state.action = Some(Action::BlockOutgoing {
        bypass: true,
        replace: true,
        timeout: timeout,
        duration: Dist {
            dist: DistType::Uniform {
                low: INFINITY,
                high: INFINITY,
            },
            start: 0.0,
            max: 0.0,
        },
        limit: Some(limit),
    });

    return state;
}

// Generate a RegulaTor relay-side machine.
fn generate_relay_machine(
    packets_per_state: f64,
    initial_rate: f64,
    decay: f64,
    threshold: f64,
) -> String {
    let mut t1 = 0.0;
    let mut keep_going = true;
    let mut num_send_states = 0;

    // Calculate number of send states
    while keep_going {
        let width = calc_interval_width(t1, packets_per_state, initial_rate, decay);
        let middle = t1 + (width / 2.0);
        let t2 = t1 + width;

        let rate = calculate_rate(middle, initial_rate, decay);
        if width == INFINITY || rate < 1.0 {
            keep_going = false;
        }

        t1 = t2;
        num_send_states += 1;
    }

    // Set up state vector
    let num_states = num_send_states + 11;
    let mut states: Vec<State> = Vec::with_capacity(num_states);

    // START states
    states.push(generate_relay_start_state());
    states.push(generate_relay_block_state());

    // BOOTSTRAP states
    states.push(generate_relay_boot_state(2, 3, 100000.0));
    states.push(generate_relay_boot_state(3, 4, 100000.0));
    states.push(generate_relay_boot_state(4, 5, 100000.0));
    states.push(generate_relay_boot_state(5, 6, 100000.0));
    states.push(generate_relay_boot_state(6, 7, 100000.0));
    states.push(generate_relay_boot_state(7, 8, 100000.0));
    states.push(generate_relay_boot_state(8, 9, 100000.0));
    states.push(generate_relay_boot_state(9, 10, 100000.0));
    states.push(generate_relay_boot_state(10, 11, 100000.0));

    // SEND_i states
    t1 = 0.0;

    for i in 0..num_send_states {
        let width = calc_interval_width(t1, packets_per_state, initial_rate, decay);
        let middle = t1 + (width / 2.0);
        let t2 = t1 + width;

        let mut rate = calculate_rate(middle, initial_rate, decay);
        let mut next_idx = i + 12;
        let curr_idx = i + 11;

        if width == INFINITY || rate < 1.0 {
            rate = 1.0;
            next_idx = STATE_END; // StateEnd
        }

        states.push(generate_relay_send_state(
            curr_idx,
            next_idx,
            packets_per_state,
            1000000.0 / rate,
            threshold,
            rate,
        ));

        t1 = t2;
    }

    // Machine construction
    let machine = Machine {
        allowed_padding_packets: u64::MAX,
        max_padding_frac: 0.0,
        allowed_blocked_microsec: u64::MAX,
        max_blocking_frac: 0.0,
        states: states,
    };

    return machine.serialize();
}

// Generate a SEND state for a relay-side machine.
fn generate_relay_send_state(
    curr_index: usize,
    next_index: usize,
    padding_count: f64,
    timeout: f64,
    threshold: f64,
    rate: f64,
) -> State {
    // SEND_i state
    let mut state;

    if curr_index > 11 {
        state = State::new(enum_map! {
            Event::PaddingSent => vec![Trans(curr_index, 1.0)],
            Event::LimitReached => vec![Trans(next_index, 1.0)],
            Event::NormalSent => vec![Trans(11, 2.0 / (threshold * rate) as f32)],
            _ => vec![],
        });
    } else {
        state = State::new(enum_map! {
            Event::PaddingSent => vec![Trans(curr_index, 1.0)],
            Event::LimitReached => vec![Trans(next_index, 1.0)],
            _ => vec![],
        });
    }

    let timeout = Dist::new(
        DistType::Uniform {
            low: timeout,
            high: timeout,
        },
        0.0,
        0.0,
    );

    let limit = Dist::new(
        DistType::Uniform {
            low: padding_count,
            high: padding_count,
        },
        0.0,
        0.0,
    );

    state.action = Some(Action::SendPadding {
        bypass: true,
        replace: true,
        timeout: timeout,
        limit: Some(limit),
    });

    return state;
}

// Generate a BOOT state for a relay-side machine.
fn generate_relay_boot_state(curr_index: usize, next_index: usize, timeout: f64) -> State {
    let mut state = State::new(enum_map! {
        Event::PaddingSent => vec![Trans(curr_index, 1.0)],
        Event::NormalSent => vec![Trans(next_index, 1.0)],
        _ => vec![],
    });

    let timeout = Dist::new(
        DistType::Uniform {
            low: timeout,
            high: timeout,
        },
        0.0,
        0.0,
    );

    state.action = Some(Action::SendPadding {
        bypass: true,
        replace: true,
        timeout: timeout,
        limit: None,
    });

    return state;
}

// Generate the BLOCK state for a relay-side machine.
fn generate_relay_block_state() -> State {
    // BlockingBegin --> BOOT_0 (100%)
    let mut state = State::new(enum_map! {
        Event::BlockingBegin => vec![Trans(2, 1.0)],
        _ => vec![],
    });

    let duration = Dist::new(
        DistType::Uniform {
            low: INFINITY,
            high: INFINITY,
        },
        0.0,
        0.0,
    );

    let timeout = Dist::new(
        DistType::Uniform {
            low: 0.0,
            high: 0.0,
        },
        0.0,
        0.0,
    );

    state.action = Some(Action::BlockOutgoing {
        bypass: true,
        replace: true,
        timeout: timeout,
        duration: duration,
        limit: None,
    });

    return state;
}

// Generate the START state for a machine.
fn generate_relay_start_state() -> State {
    // NonPaddingSent --> BLOCK (100%)
    let state = State::new(enum_map! {
        Event::NormalSent => vec![Trans(1, 1.0)],
        _ => vec![],
    });

    return state;
}

// Find the width of an interval of the function RD^t, from a, with the specified packet count.
fn calc_interval_width(a: f64, count: f64, rate: f64, decay: f64) -> f64 {
    let mut mid = a;
    let mut step: f64 = 0.5;
    let mut decreasing = false;

    let mut curr_count = 0.0;
    let mut curr_diff = count - curr_count;

    while curr_diff.abs() > 0.00001 {
        if curr_diff < 0.0 {
            mid -= step;
            decreasing = true;
        } else {
            mid += step;
        }

        if decreasing {
            step /= 2.0;
        } else {
            step *= 2.0;
        }

        curr_count = calculate_rate(mid, rate, decay) * (mid - a) * 2.0;
        curr_diff = count - curr_count;
    }

    return (mid - a) * 2.0;
}

// RD^t
fn calculate_rate(t: f64, initial_rate: f64, decay: f64) -> f64 {
    return initial_rate * decay.powf(t);
}
