#!/bin/bash
cargo update --manifest-path=$1/Cargo.toml &&
sbatch --job-name=$2 $3/eval_speed_1.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_2.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_3.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_10.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_11.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_12.sh $1 $3 $2
