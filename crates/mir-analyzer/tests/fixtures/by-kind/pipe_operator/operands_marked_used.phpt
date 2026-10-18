===description===
Variables read on either side of `|>` count as used, and a function reached
only through a pipe is not unused.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
  <findUnusedVariables>true</findUnusedVariables>
</mir>
===file===
<?php
function onlyPiped(int $x): int { return $x + 1; }

function run(): int {
    $value = 1;
    $step = fn(int $x): int => $x * 3;
    return $value |> onlyPiped(...) |> $step;
}

echo run();
