===description===
`++$a[$k]` and `$a[$k]--` write the new value back to the element
===file===
<?php
function f(string $k): bool {
    $counts = ['low' => 0, 'high' => 0];
    ++$counts['low'];
    $counts[$k]++;
    return 1 === $counts['low'] || 1 === $counts['high'];
}
