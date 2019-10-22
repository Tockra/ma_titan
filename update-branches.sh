#!/bin/bash
git checkout master &&
git pull &&
git checkout space_efficient_space &&
git merge --no-edit space_efficient &&
git push &&
git checkout fnv_hash_space && 
git merge --no-edit fnv_hash &&
git push &&
git checkout rustc_hash_hash_space && 
git merge --no-edit rustc_hash_hash &&
git push &&
git checkout hash_brown_hash_space && 
git merge --no-edit hash_brown_hash &&
git push 
 