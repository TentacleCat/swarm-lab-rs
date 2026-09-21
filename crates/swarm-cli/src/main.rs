use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs::File;
use std::path::PathBuf;
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::{Swarmalator2D, SwarmalatorRing1D};
use swarm_core::types::DynamicalSystem;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum ModelType {
    /// 2D Swarmalator model (O'Keeffe, Hong & Strogatz, Nature Comms 2017)
    #[value(name = "2d")]
    TwoD,
    /// 1D Swarmalator model on a Ring (PRE 2018 / PRE 2022)
    #[value(name = "ring")]
    Ring,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "CLI simulator for Swarmalator dynamical systems", long_about = None)]
struct Args {
    /// Model to simulate
    #[arg(short, long, value_enum, default_value_t = ModelType::TwoD)]
    model: ModelType,

    /// Number of swarmalator agents
    #[arg(short = 'n', long, default_value_t = 100)]
    num_agents: usize,

    /// Spatial coupling parameter J
    #[arg(short = 'j', long, allow_hyphen_values = true, default_value_t = 0.1)]
    j: f64,

    /// Phase coupling parameter K
    #[arg(short = 'k', long, allow_hyphen_values = true, default_value_t = 1.0)]
    k: f64,

    /// Time step size dt
    #[arg(long, default_value_t = 0.05)]
    dt: f64,

    /// Total simulation steps
    #[arg(short = 's', long, default_value_t = 2000)]
    steps: usize,

    /// Record snapshot every N steps
    #[arg(long, default_value_t = 10)]
    sample_interval: usize,

    /// CSV output path for particle states
    #[arg(short, long, default_value = "data/trajectory.csv")]
    output: PathBuf,

    /// Optional CSV output path for order parameters time series
    #[arg(long, default_value = "data/metrics.csv")]
    metrics_output: PathBuf,

    /// Random seed
    #[arg(long, default_value_t = 42)]
    seed: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("============================================================");
    println!(" 🌌 Swarm-Lab-RS: Swarmalator Simulation Suite");
    println!("============================================================");
    println!(" Model        : {:?}", args.model);
    println!(" Agents (N)   : {}", args.num_agents);
    println!(" J (Spatial)  : {}", args.j);
    println!(" K (Phase)    : {}", args.k);
    println!(" dt           : {}", args.dt);
    println!(" Total steps  : {}", args.steps);
    println!(" Trajectory   : {}", args.output.display());
    println!(" Metrics      : {}", args.metrics_output.display());
    println!("============================================================");

    // Ensure parent directories exist
    if let Some(parent) = args.output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = args.metrics_output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut rng = StdRng::seed_from_u64(args.seed);

    match args.model {
        ModelType::TwoD => {
            let model = Swarmalator2D::new(args.num_agents, args.j, args.k);
            let mut state = model.random_initial_state(&mut rng);
            run_simulation(&model, &mut state, &args, "x,y,theta")?;
        }
        ModelType::Ring => {
            let model = SwarmalatorRing1D::new(args.num_agents, args.j, args.k);
            let mut state = model.random_initial_state(&mut rng);
            run_simulation(&model, &mut state, &args, "phi,theta")?;
        }
    }

    println!("\n✅ Simulation completed successfully!");
    println!("💡 Run python script to visualize: python3 python/plot_phases.py --model {:?}", args.model);
    Ok(())
}

fn run_simulation<S: DynamicalSystem>(
    system: &S,
    state: &mut [f64],
    args: &Args,
    coord_labels: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut integrator = Rk4Integrator::new(system.dimension());
    let mut traj_writer = csv::Writer::from_writer(File::create(&args.output)?);
    let mut metric_writer = csv::Writer::from_writer(File::create(&args.metrics_output)?);

    // Write CSV Headers
    traj_writer.write_record(&["step", "time", "agent_id", "c1", "c2", "phase"])?;
    metric_writer.write_record(&["step", "time", "order_r", "order_s", "order_s_plus", "order_s_minus"])?;

    let pb = ProgressBar::new(args.steps as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let n = system.num_agents();

    for step in 0..=args.steps {
        let current_time = (step as f64) * args.dt;

        if step % args.sample_interval == 0 {
            // Write metrics
            let mut snap = system.metrics(state);
            snap.t = current_time;
            metric_writer.write_record(&[
                step.to_string(),
                format!("{:.4}", current_time),
                format!("{:.5}", snap.order_r),
                snap.order_s.map(|v| format!("{:.5}", v)).unwrap_or_default(),
                snap.order_s_plus.map(|v| format!("{:.5}", v)).unwrap_or_default(),
                snap.order_s_minus.map(|v| format!("{:.5}", v)).unwrap_or_default(),
            ])?;

            // Write trajectory coordinates
            if coord_labels == "x,y,theta" {
                for i in 0..n {
                    let x = state[i];
                    let y = state[n + i];
                    let theta = state[2 * n + i];
                    traj_writer.write_record(&[
                        step.to_string(),
                        format!("{:.4}", current_time),
                        i.to_string(),
                        format!("{:.5}", x),
                        format!("{:.5}", y),
                        format!("{:.5}", theta),
                    ])?;
                }
            } else if coord_labels == "phi,theta" {
                for i in 0..n {
                    let phi = state[i];
                    let theta = state[n + i];
                    traj_writer.write_record(&[
                        step.to_string(),
                        format!("{:.4}", current_time),
                        i.to_string(),
                        format!("{:.5}", phi),
                        "".to_string(),
                        format!("{:.5}", theta),
                    ])?;
                }
            }
        }

        if step < args.steps {
            integrator.step(system, state, args.dt);
            pb.inc(1);
        }
    }

    pb.finish_with_message("Done");
    traj_writer.flush()?;
    metric_writer.flush()?;
    Ok(())
}
