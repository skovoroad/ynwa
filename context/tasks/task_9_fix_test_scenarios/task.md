After the game serialization tasks were implemented, it became possible to play test scenarios in the player and visually track their correct execution. For example, the command that plays the corner-handling check scenario:

./replay.sh /home/lipkin/.ynwa/scenarios/corner.jsonl

It was expected that recording and playing the scenario would fully reproduce the game situation. That is, when playing the scenario through the player, the developer would observe on screen the initial player positioning (elementary, one player per team), the ball going out over the goal line, and the test stopping (the corner took place).

In practice, the scenario finishes virtually instantly, and it is impossible to analyze its correctness visually.

This situation must be fixed.
