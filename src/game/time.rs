pub(crate) fn time_display(points: u32, day: u32) -> String {
    const PORTIONS: [&str; 12] = [
        "Deep Night",
        "Before Dawn",
        "Dawn",
        "Morning",
        "Late Morning",
        "High Sun",
        "Afternoon",
        "Late Afternoon",
        "Dusk",
        "Evening",
        "Night",
        "Midnight",
    ];
    const WIDTH: usize = 23;
    let slot = (points % 12) as usize;
    let label = PORTIONS[slot];
    let mut top = vec![' '; WIDTH];
    let mut bottom = vec![' '; WIDTH];
    let place = |line: &mut Vec<char>, idx: usize, ch: char| {
        if idx < line.len() {
            line[idx] = ch;
        }
    };
    match slot {
        0 => place(&mut bottom, 20, '~'),
        1 => place(&mut bottom, 16, '~'),
        2 => place(&mut top, 16, 'o'),
        3 => place(&mut top, 13, 'o'),
        4 => place(&mut top, 10, 'o'),
        5 => place(&mut top, 7, 'o'),
        6 => place(&mut top, 4, 'o'),
        7 => place(&mut bottom, 4, 'o'),
        8 => place(&mut bottom, 7, '~'),
        9 => place(&mut bottom, 10, '~'),
        10 => place(&mut bottom, 13, '~'),
        11 => place(&mut bottom, 16, '~'),
        _ => unreachable!(),
    }
    let top: String = top.into_iter().collect();
    let bottom: String = bottom.into_iter().collect();
    let indicator = format!("E{}W", "=".repeat(WIDTH - 2));
    format!(
        "{}\n{}\n{}  Day {} | {}",
        top, bottom, indicator, day, label
    )
}


/// Format the current two-hour world-time portion as a 24-hour clock readout.
///
/// The cycle starts at 02:00 in slot zero and returns to 00:00 at the
/// "Midnight" slot (eleven), matching the existing named world-time portions.
pub(crate) fn clock_display(points: u32) -> String {
    let hour = ((points % 12 + 1) % 12) * 2;
    format!("{hour:02}:00")
}
