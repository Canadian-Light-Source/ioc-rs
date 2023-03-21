macro_rules! tick {
    () => {
        "✔".green()
    };
}
pub(crate) use tick;

macro_rules! cross {
    () => {
        "✘".red().bold()
    };
}
pub(crate) use cross;

macro_rules! exclaim {
    () => {
        "!".yellow().bold()
    };
}
pub(crate) use exclaim;
