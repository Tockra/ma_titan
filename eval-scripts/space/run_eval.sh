#!/bin/bash
cargo update --manifest-path=$7/Cargo.toml &&
cargo build --release --manifest-path=$7/Cargo.toml &&
for ((i=$5;i<=$6;i++)) 
do
	if [ "$8" = "sbatch" ]; then
		sbatch --job-name=$2 $1/$2/$3/$i.sh $7 $4 $2
	else
		$1/$2/$3/$i.sh $7 $4 $2
	fi
done
