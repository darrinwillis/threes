use super::agent_runner;
use super::agent_runner::Agent;
use super::utils;

struct GenerationResult {
    scores: Vec<i32>,
}

pub struct TrainResult<'a, A: Agent> {
    generations: Vec<GenerationResult>,
    agent: &'a A,
}

pub fn train_agent_from_scratch<A: Agent>(agent: &mut A) -> TrainResult<A> {
    let num_generations = 10;
    let num_episodes_per_gen = 1000;
    let mut rng = utils::resolve_rng_from_seed(None);

    let mut generations = Vec::new();
    for _gen in 0..num_generations {
        let mut scores = Vec::new();
        for _episode in 0..num_episodes_per_gen {
            let result = agent_runner::play_game(Some(&mut rng), agent);
            let score = result.score;
            scores.push(score);
        }
        generations.push(GenerationResult { scores });
    }
    TrainResult { generations, agent }
}

pub fn analyze_report<A: Agent>(train_result: TrainResult<A>) {
    let mean_per_gen = train_result
        .generations
        .iter()
        .map(|tr| tr.scores.iter().sum::<i32>() as f32 / tr.scores.len() as f32)
        .collect::<Vec<f32>>();
    println!("Gen   |   mean score  | diff");
    let mut diffs = mean_per_gen
        .as_slice()
        .windows(2)
        .map(|ms| ms[1] - ms[0])
        .collect::<Vec<f32>>();
    diffs.insert(0, 0.0); // the first gen has no diff
    for (gen, mean) in mean_per_gen.iter().enumerate() {
        println!("{:>5} |{:>8} | {:>5}", gen, mean, diffs[gen]);
    }
    println!(
        "avg diff {}",
        diffs.iter().sum::<f32>() - diffs.len() as f32
    );
    train_result.agent.print();
    let mut _histogram = histogram::Histogram::new();
}
