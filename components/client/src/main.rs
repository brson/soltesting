use anyhow::{anyhow, bail, Context, Result};
use clap::{ArgEnum, Parser};
use common::TransferInstruction;
use log::info;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .filter_module("solana_client::rpc_client", log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    let config = load_config()?;
    let client = connect(&config)?;

    let program_keypair = load_program_keypair(&client, PROGRAM_KEYPAIR_FILE)?;
    println!("program id: {:#?}", program_keypair.pubkey());

    let new_keypair = Keypair::new();
    println!("new_keypair_pubkey: {:?}", new_keypair.pubkey());

    let args = Cli::parse();
    match args.cmd {
        Command::CreateAccount => {
            // example: use Solana sdk to call system_instruction directly
            let instr = system_instruction::create_account(
                &config.keypair.pubkey(),
                &new_keypair.pubkey(),
                1_000_000,
                0,
                &system_program::ID,
            );

            let blockhash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(
                &[instr],
                Some(&config.keypair.pubkey()),
                &[&config.keypair, &new_keypair],
                blockhash,
            );

            let sig = client.send_and_confirm_transaction(&tx)?;
            println!("sig: {}", sig);
        }
        Command::Transfer => {
            // example: use our onchain program
            let instr = TransferInstruction::build_instruction(
                &program_keypair.pubkey(),
                &config.keypair.pubkey(),
                &new_keypair.pubkey(),
                2_000_000,
            )?;

            let blockhash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(
                &[instr],
                Some(&config.keypair.pubkey()),
                &[&config.keypair],
                blockhash,
            );

            let sig = client.send_and_confirm_transaction(&tx)?;
            println!("sig: {}", sig);
        }
    };

    Ok(())
}

#[derive(Parser, Debug)]
struct Cli {
    #[clap(arg_enum)]
    cmd: Command,
}

#[derive(ArgEnum, Debug, Clone)]
enum Command {
    CreateAccount,
    Transfer,
}

static DEPLOY_PATH: &str = "target/deploy";
static PROGRAM_KEYPAIR_FILE: &str = "program-keypair.json";

struct Config {
    json_rpc_url: String,
    keypair: Keypair,
}

fn load_config() -> Result<Config> {
    let config_file = solana_cli_config::CONFIG_FILE
        .as_ref()
        .ok_or_else(|| anyhow!("config file path"))?;
    let cli_config = solana_cli_config::Config::load(config_file)?;
    let json_rpc_url = cli_config.json_rpc_url;
    let keypair = read_keypair_file(&cli_config.keypair_path).map_err(|e| anyhow!("{}", e))?;

    Ok(Config {
        json_rpc_url,
        keypair,
    })
}

fn connect(config: &Config) -> Result<RpcClient> {
    info!("connecting to solana node at {}", config.json_rpc_url);
    let client =
        RpcClient::new_with_commitment(config.json_rpc_url.clone(), CommitmentConfig::confirmed());

    let version = client.get_version()?;
    info!("RPC version: {:?}", version);

    Ok(client)
}

fn load_program_keypair(client: &RpcClient, program_keypair_file: &str) -> Result<Keypair> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let deploy_path = format!("{}/../../{}", manifest_dir, DEPLOY_PATH);
    let program_keypair_path = format!("{}/{}", deploy_path, program_keypair_file);

    info!("loading program keypair from {}", program_keypair_path);

    let program_keypair = read_keypair_file(&program_keypair_path)
        .map_err(|e| anyhow!("{}", e))
        .context("unable to load program keypair")?;

    let program_id = program_keypair.pubkey();

    info!("program id: {}", program_id);

    let account = client
        .get_account(&program_id)
        .context("unable to get program account")?;

    info!("program account: {:?}", account);

    if !account.executable {
        bail!("solana account not executable");
    }

    Ok(program_keypair)
}
