#!/bin/bash
cargo update --manifest-path=$1/Cargo.toml &&
sbatch --job-name=$2 $3/eval_space_1.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_2.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_3.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_4.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_5.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_6.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_7.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_8.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_9.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_10.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_11.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_13.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_14.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_16.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_space_17.sh $1 $3 $2
