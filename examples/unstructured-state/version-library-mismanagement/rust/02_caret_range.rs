struct Release {
    major: u32,
    minor: u32,
    patch: u32,
    leap_february: bool,
}

fn accepts_caret(found: &Release) -> bool {
    found.major == 1 && found.minor >= 2
}

fn february_days(year: i32, release: &Release) -> i32 {
    if release.leap_february && year % 4 == 0 {
        29
    } else {
        28
    }
}

fn main() {
    let published = [
        Release {
            major: 1,
            minor: 2,
            patch: 0,
            leap_february: true,
        },
        Release {
            major: 1,
            minor: 9,
            patch: 0,
            leap_february: false,
        },
    ];
    let mut chosen = &published[0];
    for release in &published {
        if accepts_caret(release) && release.minor > chosen.minor {
            chosen = release;
        }
    }
    let year = 2024;
    println!(
        "february {year}: {} via {}.{}.{}",
        february_days(year, chosen),
        chosen.major,
        chosen.minor,
        chosen.patch
    );
}
