import os
import subprocess
import time
import json


import streamlit as st
import pandas as pd
import plotly.express as px


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
        for game in self._outcomes["games_played"]:
            assert set(game.keys()) == {"score", "gen_id"}

    def as_dataframe(self):
        return pd.DataFrame.from_records(self.d()["games_played"])


def run_training(retrain=False):
    kResultPath = "train_results.json"

    retrain = retrain or not os.path.exists(kResultPath)

    if retrain:
        start = time.time()
        result = subprocess.run(["cargo", "run", "--release", "--", "train"])
        result.check_returncode()
        train_time = time.time() - start
        st.write(f"Trained new agent in {train_time:0.2f}s")
    else:
        st.write("Loading pretrained agent")

    with open(kResultPath) as f:
        result_dict = json.load(f)

    outcomes = TrainingOutcomes(result_dict)

    st.write(f"Loaded agent with {outcomes.num_generations()} generations; {outcomes.num_games()} games")

    return outcomes

def draw_top_summary(outcomes):
    df = outcomes.as_dataframe()

    fig = px.scatter(df, title="Scores across training", y="score", color="gen_id")
    st.write(fig)


    window_size = 500

    rolling_avg = df.rolling(window_size).mean()

    fig = px.line(rolling_avg, title="Scores across training (windowed)", y="score")
    st.write(fig)

    quantiles = [.1, .5, .9, 1]
    # Calculate the quantiles as grouped by generation
    quantiles_by_gen = df.groupby(["gen_id"]).quantile(quantiles)

    # Rename the quantile columns
    quantiles_by_gen = quantiles_by_gen.unstack().score.rename(columns = lambda c: f"p{int(100*float(c))} score")

    fig = px.line(quantiles_by_gen, title="Quantiles by training generation")
    st.write(fig)


outcomes= run_training()
draw_top_summary(outcomes)
