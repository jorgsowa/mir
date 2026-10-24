===description===
A pipe chain threads each step's return type into the next call.
===file===
<?php
function double(int $x): int { return $x * 2; }

$len = "  padded  "
    |> trim(...)
    |> strlen(...)
    |> double(...)
    |> (fn(int $n): string => str_repeat('-', $n));
/** @trace $len */
echo $len;
//<^^^^^^^^^^ Trace: Type of $len is string
