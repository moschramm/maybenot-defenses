// Maybenot FRONT -- uses normally distributed padding to approximate the FRONT defense
// Code from the paper "State Machine Frameworks for Website Fingerprinting Defenses: Maybe Not"

use enum_map::enum_map;
use std::env;
use std::f64::consts::E;
use std::f64::consts::PI;
use std::f64::EPSILON;

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
        args.len() == 4,
        "Usage: {} <padding window> <padding budget> <num states>",
        &args[0]
    );

    let padding_window: f64 = args[1].parse().expect("Invalid padding window"); // FRONT param = W_max (sec)
    let padding_budget: u32 = args[2].parse().expect("Invalid padding budget"); // FRONT param = N (num cells)
    let num_states: u32 = args[3].parse().expect("Invalid num states"); // number of PADDING states

    let machine = generate_machine(
        padding_window * 1000000.0,
        padding_budget,
        num_states as usize,
    );
    println!("Machine: {} ({})\n", machine, machine.len());
}

// Generate a FRONT machine with the specified number of PADDING states.
fn generate_machine(padding_window: f64, padding_budget: u32, num_states: usize) -> String {
    let area = 1.0 / (num_states as f64); // Area under Rayleigh CDF curve of each state
    let max_t = rayleigh_max_t(padding_window);

    // States
    let mut states: Vec<State> = Vec::with_capacity(num_states + 1);
    states.push(generate_start_state());

    let mut t1 = 0.0; // Starting time of next PADDING state
    let mut total_padding_frac = 0.0; // Area coverage of current PADDING states

    for i in 1..num_states {
        let width = calc_interval_width(t1, max_t, area, padding_window);
        let middle = t1 + (width / 2.0);
        let t2 = t1 + width;

        let padding_count = area * (padding_budget as f64);
        let timeout = width / padding_count;
        let stdev = (padding_window).powi(2) / (padding_count * middle * PI.sqrt());

        states.push(generate_padding_state(
            i,
            i + 1,
            padding_count,
            timeout,
            stdev,
        ));

        t1 = t2;
        total_padding_frac += area;
    }

    // Last state, to max_t
    let width = max_t - t1;
    let middle = t1 + (width / 2.0);

    let padding_count = (1.0 - total_padding_frac) * (padding_budget as f64);
    let timeout = width / padding_count;
    let stdev = (padding_window).powi(2) / (padding_count * middle * PI.sqrt());

    // add last padding state
    states.push(generate_last_padding_state(
        num_states,
        padding_count,
        timeout,
        stdev,
    ));

    // Machine
    let machine = Machine {
        allowed_padding_packets: u64::MAX,
        max_padding_frac: 0.0,
        allowed_blocked_microsec: 0,
        max_blocking_frac: 0.0,
        states: states,
    };

    return machine.serialize();
}

// Generate a PADDING state for a machine.
fn generate_padding_state(
    curr_index: usize,
    next_index: usize,
    padding_count: f64,
    timeout: f64,
    stdev: f64,
) -> State {
    let mut state = State::new(enum_map! {
        Event::PaddingSent => vec![Trans(curr_index, 1.0)],
        Event::LimitReached => vec![Trans(next_index, 1.0)],
        _ => vec![],
    });

    let timeout = Dist::new(
        DistType::Normal {
            mean: timeout,
            stdev: stdev,
        },
        0.0,
        timeout * 2.0,
    );
    let limit = Dist::new(
        DistType::Uniform {
            low: 1.0,
            high: padding_count,
        },
        0.0,
        0.0,
    );

    state.action = Some(Action::SendPadding {
        bypass: false,
        replace: false,
        timeout: timeout,
        limit: Some(limit),
    });

    return state;
}

// Generate the last PADDING state for a machine.
fn generate_last_padding_state(
    curr_index: usize,
    padding_count: f64,
    timeout: f64,
    stdev: f64,
) -> State {
    let mut state = State::new(enum_map! {
        Event::PaddingSent => vec![Trans(curr_index, 1.0)],
        Event::LimitReached => vec![Trans(STATE_END, 1.0)],
        _ => vec![],
    });

    let timeout = Dist::new(
        DistType::Normal {
            mean: timeout,
            stdev: stdev,
        },
        0.0,
        timeout * 2.0,
    );
    let limit = Dist::new(
        DistType::Uniform {
            low: 1.0,
            high: padding_count,
        },
        0.0,
        0.0,
    );

    state.action = Some(Action::SendPadding {
        bypass: false,
        replace: false,
        timeout: timeout,
        limit: Some(limit),
    });

    return state;
}

// Generate the START state for a machine.
fn generate_start_state() -> State {
    return State::new(enum_map! {
        Event::NormalSent => vec![Trans(1, 1.0)],
        Event::NormalRecv => vec![Trans(1, 1.0)],
        _ => vec![],
    });
}

// Find the width of an interval in the Rayleigh distribution,
// starting at a, with the specified area. Uses a search algorithm
// because numerical error affects direct calculation significantly.
fn calc_interval_width(a: f64, max_t: f64, area: f64, scale: f64) -> f64 {
    let mut b = max_t;
    let mut increment = (b - a) / 2.0;

    let mut curr_area = rayleigh_cdf(b, scale) - rayleigh_cdf(a, scale);
    let mut curr_diff = area - curr_area;

    while curr_diff.abs() > EPSILON {
        if curr_diff < 0.0 {
            b -= increment;
        } else {
            b += increment;
        }
        increment /= 2.0;

        curr_area = rayleigh_cdf(b, scale) - rayleigh_cdf(a, scale);
        curr_diff = area - curr_area;
    }

    return b - a;
}

// Cumulative distribution function of Rayleigh distribution
fn rayleigh_cdf(t: f64, scale: f64) -> f64 {
    let exp_num = -t.powi(2);
    let exp_div = 2.0 * scale.powi(2);
    let exp = exp_num / exp_div;

    return 1.0 - E.powf(exp);
}

// Return the value of t (input to Rayleigh CDF) at which area = 0.9996645373720975, chosen
// empirically. This is a bit more than 6 standard deviations.
fn rayleigh_max_t(scale: f64) -> f64 {
    let a: f64 = -2.0 * scale.powi(2);
    let b: f64 = 1.0 - 0.9996645373720975;

    return (a * b.log(E)).sqrt();
}
