#!/bin/bash
cargo update --manifest-path=$1/Cargo.toml &&
sbatch u40/eval_speed_1.sh $1 &&
sbatch u40/eval_speed_2.sh $1 &&
sbatch u40/eval_speed_3.sh $1 