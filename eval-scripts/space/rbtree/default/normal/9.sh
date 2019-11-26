#!/bin/bash -l
#SBATCH --time=00:30:00      # The maximum wall time of this job is two hours.
#SBATCH --partition=short    
#SBATCH --nodes=1          
#SBATCH --constraint=cstd01 # ... with 48 cores and 256 GB RAM ...
#SBATCH --mem=62250         # ... of which we use 250 GB.
#SBATCH --mail-user=tim.tannert@tu-dortmund.de
#SBATCH --mail-type=FAIL
#SBATCH --exclusive          # In addition, we want to be the only user on the node.
#SBATCH --ntasks-per-node=1 # We do not want to have more than 48 tasks running on the node,
#SBATCH --cpus-per-task=20    # and we want each task running on its own (exclusive) core.
#SBATCH --output=/work/smtitann/output/space/rbtree_normal_9_%j.dat # The location, where all output is collected. %j will be replaced with the job id.
cd $1 # Go to the directory, where the executable is stored.
cargo run --release rbtree $2 normal 9 $3