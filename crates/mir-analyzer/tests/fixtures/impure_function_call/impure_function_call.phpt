===description===
ImpureFunctionCall
===file===
<?php
/**
 * @template T
 *
 * @param array<int, T> $values
 * @param (callable(T): numeric) $num_func
 *
 * @return null|T
 *
 * @pure
 */
function max_by(array $values, callable $num_func)
{
    $max = null;
    $max_num = null;
    foreach ($values as $value) {
        $value_num = $num_func($value);
        if (null === $max_num || $value_num >= $max_num) {
            $max = $value;
            $max_num = $value_num;
        }
    }

    return $max;
}

$c = max_by([1, 2, 3], static function(int $a): int {
    return $a + mt_rand(0, $a);
});

echo $c;

===expect===
ImpureFunctionCall
===ignore===
TODO
