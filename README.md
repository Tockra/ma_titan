# PredecessorMA

Dieses Repository wird die Implementierung der Predecessor-Datenstruktur basierend auf http://algo2.iti.kit.edu/dementiev/files/veb.pdf .
Die Implementierung geschieht in Rust und kann mit 40-Bit-,48-Bit- und 64-Bit-Schlüsseln umgehen.

Die verschiedenen Branches implementieren die verschiedenen in der Abschlussarbeit vorgestellten STree-Implementierungen. 
Achtung: Die Kommentare der 48- und 64-Bit-Implementierungen wurden nicht angepasst und sind zu ignorieren!

In ./eval-scripts liegen Skripte, die bei der Evaluierung mittels ma_eval_speed und ma_eval_space helfen. 



Die Branches können wie folgt zugeordnet werden:

40-Bit:
| Branch        | Bezeichnugn in der Arbeit (Evaluierung)|
| -------------------- |---------------------------------| 
| master        | mphf_2 |
| mphf_16       | mphf_1      |
| original      | original_2      | 
| original_16   | original_1      | 
| lookup        | lookup_2      | 
| lookup_16     | lookup_1      | 
| threshold     | threshold_2      | 
| threshold_16  | threshold_1     | 
| space_efficient |  binary_2      | 
| space_efficient_16 | binary_1      | 
| fnv_hash      | fnv_2      | 
| fnv_hash_16        | fnv_1      | 
| hash_brown_hash| ahash_2      | 
| brown_hash_16 | ahash_1      | 

48-Bit:
| Branch        | Bezeichnugn in der Arbeit (Evaluierung)|
| ------------------ |----------------------------------------| 
| mphf_u48_1       | mphf_1     |
| mphf_u48_2       | mphf_2     |
| original_u48_1   | original_1 | 
| original_u48_2   | original_2 | 
| lookup_u48_1     | lookup_1   | 
| lookup_u48_2  | lookup_2   | 
| threshold_u48_1  | threshold_1| 
| threshold_u48_2  | threshold_2| 
| space_efficient_u48_1  |  binary_1      | 
| space_efficient_u48_2  | binary_2      | 
| fnv_hash_u48_1   | fnv_1      | 
| fnv_hash_u48_2   | fnv_2      | 
| brown_hash_u48_1 | ahash_1    | 
| brown_hash_u48_2 | ahash_2    | 

64-Bit:

| Branch                 | Bezeichnugn in der Arbeit (Evaluierung)|
| ---------------------- |----------------------------------------| 
| mphf_u64_1             | mphf_1                                 |
| mphf_u64_2             | mphf_2                                 |
| original_u64_1         | original_1                             | 
| original_u64_2         | original_2                             | 
| lookup_u64_1           | lookup_1                               | 
| lookup_u64_2           | lookup_2                               | 
| threshold_u64_1        | threshold_1                            | 
| threshold_u64_2        | threshold_2                            | 
| space_efficient_u64_1  |  binary_1                              | 
| space_efficient_u64_2  | binary_2                               | 
| fnv_hash_u64_1         | fnv_1                                  | 
| fnv_hash_u64_2         | fnv_2                                  | 
| brown_hash_u64_1       | ahash_1                                | 
| brown_hash_u64_2       | ahash_2                                | 


| First Header  | Second Header |
| ------------- | ------------- |
| Content Cell  | Content Cell  |
| Content Cell  | Content Cell  |
