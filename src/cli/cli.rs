use clap::Parser;
use std::path::PathBuf;

use crate::transform::transformer::TransformType;

#[derive(Parser)]
#[clap(name = "git")]
#[clap(about = "A fictional versioning CLI", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Parser)]
#[clap(name = "rfood")]
#[clap(bin_name = "rfood")]
pub enum Commands {
    #[clap()]
    PrintTest,
    #[clap(arg_required_else_help = true)]
    Transform{
        /// The path of the file to transform
        #[clap(required = true, parse(from_os_str))]
        path: PathBuf,
        #[clap(arg_enum, default_value_t = TransformType::OOPToFP)]
        transform_type: TransformType,
    },
}
