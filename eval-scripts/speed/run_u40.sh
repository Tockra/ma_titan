#!/bin/bash
cargo update --manifest-path=$1/Cargo.toml &&
sbatch --job-name=$2 $3/eval_speed_1.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_2.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_3.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_4.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_5.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_6.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_7.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_8.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_9.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_10.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_11.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_12.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_13.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_14.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_15.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_16.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_17.sh $1 $3 $2 &&
sbatch --job-name=$2 $3/eval_speed_18.sh $1 $3 $2