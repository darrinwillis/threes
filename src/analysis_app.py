import os
import subprocess
import time
import json
import hashlib


import streamlit as st
import pandas as pd
import plotly.express as px

NUM_GEN = 1000


DEBUG = False


class TrainingOutcomes:
    def __init__(self, outcomes_dict):
        self._outcomes = outcomes_dict
        self.validate()

    def d(self):
        return self._outcomes

    def num_generations(self):
        return len(set(g["gen_id"] for g in self.d()["games_played"]))
        return len(self.d()["generations"])

    def num_games(self):
        return len(self.d()["games_played"])

    def scores(self):
        return [g["score"] for g in self.d()["games_played"]]

    def validate(self):
        assert set(self._outcomes.keys()) == {"games_played"}
        for game in self.d()["games_played"]:
            assert set(game.keys()) == {"score", "gen_id", "game_log"}
        # spot check a game
        game_log = self.d()["games_played"][0]["game_log"]
        assert set(game_log.keys()) == {"seed", "moves"}
        assert set(game_log["moves"]).issubset({"Up", "Down", "Left", "Right"})

    def as_dataframe(self):
        # NOTE we are ignoring game_log
        return pd.DataFrame.from_records(
            self.d()["games_played"], columns=["score", "gen_id"]
        )


def run_training(
    retrain=False,
    learning_rate=None,
    discount_factor=None,
    explore_rate=None,
    num_generations=NUM_GEN,
    num_episodes_per_gen=100,
):

    cmdline = [
        "cargo",
        "run",
        "--release",
        "--",
        "train",
    ]

    if num_generations is not None:
        cmdline.extend(["--num_generations", str(num_generations)])

    if learning_rate is not None:
        cmdline.extend(["--learning_rate", str(learning_rate)])

    if discount_factor is not None:
        cmdline.extend(["--discount_factor", str(discount_factor)])

    if explore_rate is not None:
        cmdline.extend(["--explore_rate", str(explore_rate)])

    if num_episodes_per_gen is not None:
        cmdline.extend(["--num_episodes_per_gen", str(num_episodes_per_gen)])

    cmd_hash = hashlib.md5(" ".join(cmdline).encode("utf-8")).hexdigest()

    result_file = f"train_file_{cmd_hash}.json"

    retrain = retrain or not os.path.exists(result_file)

    if retrain:
        start = time.time()

        cmdline.extend(["--result_file", result_file])
        result = subprocess.run(cmdline)
        result.check_returncode()

        train_time = time.time() - start
        if DEBUG:
            st.write(f"Trained new agent in {train_time:0.2f}s")
    else:
        if DEBUG:
            st.write("Loading pretrained agent")

    with open(result_file) as f:
        result_dict = json.load(f)

    outcomes = TrainingOutcomes(result_dict)

    if DEBUG:
        st.write(
            f"Loaded agent with {outcomes.num_generations()} generations; {outcomes.num_games()} games"
        )

    return result_file, outcomes


def draw_top_summary(result_file, outcomes):
    st.write(f"Default trained model. Results in {result_file}")
    df = outcomes.as_dataframe()

    fig = px.scatter(df, title="Scores across training", y="score", color="gen_id")
    st.write(fig)

    window_size = 5

    rolling_avg = df.rolling(window_size).mean()

    fig = px.line(rolling_avg, title="Scores across training (windowed)", y="score")
    st.write(fig)

    quantiles = [0.1, 0.5, 0.9, 1]
    # Calculate the quantiles as grouped by generation
    quantiles_by_gen = df.groupby(["gen_id"]).quantile(quantiles)

    # Rename the quantile columns
    quantiles_by_gen = quantiles_by_gen.unstack().score.rename(
        columns=lambda c: f"p{int(100*float(c))} score"
    )

    fig = px.line(quantiles_by_gen, title="Quantiles by training generation")
    st.write(fig)


def test_explore():
    explore_values = [0.0001, 0.001, 0.01, 0.1, 0.5]

    dfs = []
    for exp in explore_values:
        _, results = run_training(explore_rate=exp)

        df = results.as_dataframe()

        df["explore_rate"] = str(exp)

        dfs.append(df)

    total_df = pd.concat(dfs)

    st.write("total df:")
    st.write(total_df)

    fig = px.scatter(
        total_df, title="Scores across training", y="score", color="explore_rate"
    )
    st.write(fig)


def test_discount():
    discount_values = [0.7, 0.9, 0.99, 0.999]

    dfs = []
    for disc in discount_values:
        fn = f"train_disc_{disc:0.2f}.json"
        _, results = run_training(discount_factor=disc)

        df = results.as_dataframe()

        df["discount_factor"] = str(disc)

        dfs.append(df)

    total_df = pd.concat(dfs)

    st.write("total df:")
    st.write(total_df)

    window_size = 5
    rolling_avg = (
        total_df.drop("gen_id", axis=1)
        .groupby(["discount_factor"])
        .rolling(window_size)
        .quantile(0.9)
    )

    rolling_avg = rolling_avg.drop("discount_factor", axis=1).reset_index(
        col_fill="age"
    )

    fig = px.line(
        rolling_avg,
        title=f"Scores across training (window_size={window_size})",
        x="level_1",
        y="score",
        color="discount_factor",
    )
    st.write(fig)

    fig = px.scatter(
        total_df, title="Scores across training", y="score", color="discount_factor"
    )
    st.write(fig)


result_file, outcomes = run_training()
draw_top_summary(result_file, outcomes)

test_explore()

test_discount()
