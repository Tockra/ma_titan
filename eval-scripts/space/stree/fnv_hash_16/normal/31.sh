#!/bin/bash -l
#SBATCH --time=08:00:00      # The maximum wall time of this job is two hours.
#SBATCH --partition=med    
#SBATCH --nodes=1          
#SBATCH --constraint=cquad01 # ... with 48 cores and 256 GB RAM ...
#SBATCH --mem=250000         # ... of which we use 250 GB.
#SBATCH --mail-user=tim.tannert@tu-dortmund.de
#SBATCH --mail-type=FAIL
#SBATCH --exclusive          # In addition, we want to be the only user on the node.
#SBATCH --ntasks-per-node=1 # We do not want to have more than 48 tasks running on the node,
#SBATCH --cpus-per-task=48    # and we want each task running on its own (exclusive) core.
#SBATCH --output=/work/smtitann/output/space/stree_%x_normal_31_%j.dat # The location, where all output is collected. %j will be replaced with the job id.
cd $1 # Go to the directory, where the executable is stored.
cargo run --release stree $2 normal 31 $3

