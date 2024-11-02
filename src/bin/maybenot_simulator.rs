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

    // Load the machine from a string
    let m = "02eNqV2AlQVHUcwPFdEHZZlktwuUEQQUTlUANE5cdgooVgiOMgeDEe00ha2lRCxmWojVIKiaDYhGFOqCmNpJMj9ZajZTni3DiEQMApPFggTpHy/en3/Ds10//NMMN+mT+f2f97//++fZNT04fo5WPx3z9isVgkSgp84Zfp4/lLsehZ/s7YhD7QKcjLUU34BuFf+T4B4umX/dPj1oLoXw/hn4vFOv8wvDB5MLvwwnwNlNZsKbCsmYMC6Q8YBB0UdCnh/IYHbdtM6kBvUOaeGeiAAuldDIIuCjMoYfHGwDSVthrG6t56ErnVGgXS2xmEGSjoUcLwTd0zFUVq+CNZujw4QIEC6a0Mgh4K+pQQUzMQoI5XgSI5pvjNSXMUSG9mEPRRkFCCZtBnWVhTOQT5Np58FDITBdI1DIIEBSklVK3I8v4pswy+l304urvPFAXSmxgEKQoGlLBm7yfawLOlYCvrd3o/2wQF0hsZBAMUZJQgyrkStae6BKLfjWpYXWQkrDi+NzAIMhQMKeGh4cchAR4lEFqlbve0lKNAOotgiIKcEpR7jWWnLykhsOOSjf5VGQqk1zMIchSM6DNtFRNdsEwJd4fWpg7FGQhnmu8sghEKxpTg2JHz7db7HBwKeT1BJ1yKAul1DIIxCiaU8On1BZHncjlY/+sJjcc6CQqkswgmKJhSwjv+iWGWeziYe7y/qTJWHwXSWQRTFMwoIeG1R742KznY5JW3AU7ooUA6i2CGwkx6PUi3n4hy4kAaEeKdWzZDWA98ZxFmomBOCZ0DP8NvJhxUprV1HDURBNJZBHMULOhZ8nLtnZRycCfc1M1sl64wS3xnESxQmEUJ7qc5SYCcg5Q87WBuqQ4KpLMIs1BQ0LO0pHrdG9YcxH5xcVejpyCQziIoULCk91ZN0nuuXhzsu3VgVUaeWNhb+c4iWKJgRa9p/9FTqvUcNGka8u/ZCQLpLIIVCtaUIPvhqseBBA58d3Z3tuWKUCCdRbBGwYb+FJ26EZB8nYO1ZarARhdBIJ1FsEHB9qW9tbIztZ+D9P3HGoJjp0DYW593FsEWBTtKuHbU3txvuRIu294v2nHkGY4nnWX3tkPBnp6l8riTGZlKkKdulkxdm8TxpLMI9ig40LMUWZhVNa6EoKasCc/Op8Is8Z1FcEDBkb6Win/ZEvd2CbxS/zjK3kYQSGf5nHZEYTa9a/iNWm0aKQHv6G1hoqgJHE86izAbBSdK2J1joHp8qhQOcZFfO+SO43jSWe6XnFBwpt/DZ1ecvYLLwHjJxdDynjHhPfCd5Z7PGYU5lGDhHP5dl3E5pPiNcmE+gkA6y33rHBRc6L017fi+1rFy8Kk4e8wwZRTHk84iuKAwl14PiVUJMTIVjI+YySMaRoT1wHeWu/u5KLjS9xq3syLz1lTAj91tvfvnCQLpLQyCKwpulBBxeTD0drEadm2Xt+cfHsbxpLN8B3JDYR4l3Nqcraf+qAqSkr/pdqz/E8eTfo9BmIeCO/0tKySxetm5Gjhe2976u5sgkN7BILijMJ++liLMU+1sakGtb9jWHj8kXEt872QQ5qPgQZ+HI3A0404dFB8s70+vGBTOA99Zvk97oLCA3jXOH1L3tDTAl34Kd/NZgkB6N4OwAIWFlPBVaMdmzU0NLB11jb27aQDHk87y1GEhCovofSkz59WUsla4sXGpRVK6VtiX+N7HICxCwZMSVuw02u8u6YTD0RdaEgv7cTzpWgbBEwUv+modqV0d/nkvSPx3KOLjnghXK9+HGQQvFLzpfak57UyzUgsre2IU8X0PhX2J7+Kg/y94o+BDCQu1/nEfpFsEZXW5Tqwq7MXxpLu8IATzD8nW/Kfgg8JT8jSNKH8B7z8Nrw==";
    let m = Machine::from_str(m).unwrap();

    // Run the simulator with the machine at the client. Run the simulation up
    // until 100 packets have been recorded (total, client and server).
    let trace = sim(&[m], &[], &mut input_trace, network.delay, 100, true);

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
