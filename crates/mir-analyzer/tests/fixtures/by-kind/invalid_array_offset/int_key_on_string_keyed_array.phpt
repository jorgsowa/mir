===description===
An int offset read or destructured from an array whose keys are only non-numeric strings is an invalid offset.
===file===
<?php
/**
 * @param array<string, int> $m
 * @param array<'1'|'a', int> $numericKey
 * @param array<int|string, int> $mixedKeys
 */
function f(array $m, array $numericKey, array $mixedKeys): int
{
    $x = $m[0];
    //      ^ InvalidArrayOffset: Array offset expects 'string', got '0'
    [$a, $b] = $m;
//   ^^ InvalidArrayOffset: Array offset expects 'string', got '0'
//       ^^ InvalidArrayOffset: Array offset expects 'string', got '1'
    ['k' => $k] = $m;
    if (isset($m[1])) {
        $x++;
    }

    return $x + $a + $b + $k + $m['a'] + $numericKey[1] + $mixedKeys[0];
}
