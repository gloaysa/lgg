use clap::ValueEnum;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Style {
    Long,
    Short,
}
