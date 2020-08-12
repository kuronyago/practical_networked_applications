use project_3::Result;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "kvs-client")]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "get", about = "get-command")]
    Get,
    #[structopt(name = "set", about = "set-command")]
    Set,
    #[structopt(name = "rm", about = "rm-command")]
    Remove,
}

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        eprintln!("{}", e);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    unimplemented!()
    // match opt.command {
    //     Command::Get => {
    //         unimplemented!()
    //     },
    //     Command::Set => {
    //         unimplemented!()
    //     },
    //     Command::Remove => {
    //         unimplemented!()
    //     }
    // }
    // Ok(())
}
