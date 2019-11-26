#!/bin/bash
cargo update --manifest-path=$1/Cargo.toml &&
sbatch --job-name=$2 $3/eval_space_1.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_2.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_3.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_10.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_11.sh $1 $3 $2
