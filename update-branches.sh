#!/bin/bash
git checkout master &&
git pull &&
git checkout space_efficient_space &&
git merge --no-edit space_efficient &&
git push &&
git checkout space_efficient_max_space &&
git merge --no-edit space_efficient_space_max &&
git push &&
git checkout space_efficient_128_space &&
git merge --no-edit space_efficient_128 &&
git push 
 