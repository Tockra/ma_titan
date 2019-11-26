#!/bin/bash
cargo update --manifest-path=$8/Cargo.toml &&
cargo build --release --manifest-path=$8/Cargo.toml
for ((i=$6;i<=$7;i++)) 
do
	if [ "$9" = "sbatch" ]; then
		sbatch --job-name=$2 $3/$1/$2/$4/$i.sh $8 $5 $2
	else
		$3/$1/$2/$4/$i.sh $8 $5 $2
	fi
done
