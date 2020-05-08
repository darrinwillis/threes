use super::agent_runner;
use super::agent_runner::Agent;
use super::utils;
use std::iter;

struct GenerationResult {
    scores: Vec<i32>,
}

pub struct TrainResult<'a, A: Agent> {
    generations: Vec<GenerationResult>,
    agent: &'a A,
}

pub fn train_agent_from_scratch<A: Agent>(agent: &mut A) -> TrainResult<A> {
    let num_generations = 100;
    let num_episodes_per_gen = 1000;
    let rng = utils::resolve_rng_from_seed(None);

    let mut generations = Vec::new();
    for _gen in 0..num_generations {
        let mut scores = Vec::new();
        for _episode in 0..num_episodes_per_gen {
            // Note that we're running with the SAME game every time here
            let result = agent_runner::play_game(Some(&mut rng.clone()), agent);
            let score = result.score;
            scores.push(score);
        }
        generations.push(GenerationResult { scores });
    }
    TrainResult { generations, agent }
}

fn mean(data: &[i32]) -> Option<f32> {
    let sum = data.iter().sum::<i32>() as f32;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

fn mean_f32(data: &[f32]) -> Option<f32> {
    let sum = data.iter().sum::<f32>();
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

fn std_deviation(data: &[i32]) -> Option<f32> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as f32);

                    diff * diff
                })
                .sum::<f32>()
                / count as f32;

            Some(variance.sqrt())
        }
        _ => None,
    }
}

pub fn analyze_report<A: Agent>(train_result: TrainResult<A>) {
    let mean_per_gen = train_result
        .generations
        .iter()
        .map(|tr| mean(&tr.scores).unwrap())
        .collect::<Vec<f32>>();
    let std_dev_per_gen = train_result
        .generations
        .iter()
        .map(|tr| std_deviation(&tr.scores).unwrap())
        .collect::<Vec<f32>>();
    println!("Gen   |   mean score  | diff");
    // the first gen has no "diff"
    let first_zero = iter::once(0.0);
    let mean_differential = mean_per_gen.as_slice().windows(2).map(|ms| ms[1] - ms[0]);
    let mean_differential = first_zero.chain(mean_differential).collect::<Vec<f32>>();

    let first_zero = iter::once(0.0);
    let std_dev_differential = std_dev_per_gen
        .as_slice()
        .windows(2)
        .map(|ms| ms[1] - ms[0]);
    let std_dev_differential = first_zero.chain(std_dev_differential).collect::<Vec<f32>>();

    for (gen, (mean, mean_diff, std_dev, std_dev_diff)) in izip!(
        mean_per_gen.iter(),
        mean_differential.iter(),
        std_dev_per_gen.iter(),
        std_dev_differential.iter()
    )
    .enumerate()
    {
        println!(
            "{:>5} |{:>8} | {:>5} | {:>5} | {:>5}",
            gen, mean, mean_diff, std_dev, std_dev_diff
        );
    }
    println!(
        "avg mean diff {} avg std_dev diff {}",
        mean_f32(&mean_differential).unwrap(),
        mean_f32(&std_dev_differential).unwrap()
    );
    train_result.agent.print();
    let mut _histogram = histogram::Histogram::new();
}
