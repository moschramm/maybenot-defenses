use maybenot::{event::TriggerEvent, Machine};
use maybenot_simulator::{network::Network, parse_trace, sim};
use std::{str::FromStr, time::Duration};

fn main() {
    // A trace of ten packets from the client's perspective when visiting
    // google.com over WireGuard. The format is: "time,direction,size\n". The
    // direction is either "s" (sent) or "r" (received). The time is in
    // nanoseconds since the start of the trace. The size is in bytes.
    let raw_trace = "0,s,52
19714282,r,52
183976147,s,52
243699564,r,52
1696037773,s,40
2047985926,s,52
2055955094,r,52
9401039609,s,73
9401094589,s,73
9420892765,r,191";

    // The network model for simulating the network between the client and the
    // server. Currently just a delay.
    let network = Network::new(Duration::from_millis(10), None);

    // Parse the raw trace into a queue of events for the simulator. This uses
    // the delay to generate a queue of events at the client and server in such
    // a way that the client is ensured to get the packets in the same order and
    // at the same time as in the raw trace.
    let mut input_trace = parse_trace(raw_trace, &network);

    // Load the client machine from a string
    let cm = "02eNrFj7ENgDAMBP122AoplMxEA7swFDUbsEAEgjjEpEpFvnnLftnncKpIFcpGBwaoUsf89ax3xVC4zdyHFq9mBz8icETghggSEaQdArtxn7zQtva2ToAukQH1SE8yP3QBEOos4A==";
    let cm = Machine::from_str(cm).unwrap();

    // Load the client machine from a string
    let sm = "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";
    let sm = Machine::from_str(sm).unwrap();

    // Run the simulator with the machine at the client. Run the simulation up
    // until 100 packets have been recorded (total, client and server).
    let trace = sim(&[cm], &[sm], &mut input_trace, network.delay, 100, true);

    // print packets from the client's perspective
    let starting_time = trace[0].time;
    trace
        .into_iter()
        .filter(|p| p.client)
        .for_each(|p| match p.event {
            TriggerEvent::TunnelSent => {
                if p.contains_padding {
                    println!(
                        "sent a padding packet at {} ms",
                        (p.time - starting_time).as_millis()
                    );
                } else {
                    println!(
                        "sent a normal packet at {} ms",
                        (p.time - starting_time).as_millis()
                    );
                }
            }
            TriggerEvent::TunnelRecv => {
                if p.contains_padding {
                    println!(
                        "received a padding packet at {} ms",
                        (p.time - starting_time).as_millis()
                    );
                } else {
                    println!(
                        "received a normal packet at {} ms",
                        (p.time - starting_time).as_millis()
                    );
                }
            }
            _ => {}
        });
}
