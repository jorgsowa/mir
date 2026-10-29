===description===
Before PHP 8.4 the unpacked value's type is not named.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
function sum(int ...$numbers): int { return array_sum($numbers); }

function demo(string $text): int {
    return sum(...$text);
//             ^^^^^^^^ InvalidUnpack: Only arrays and Traversables can be unpacked
}
