===description===
Impure callable argument
===file===
<?php
/**
 * @param array<int, int> $values
 * @param pure-callable(int):int $num_func
 *
 * @pure
 */
function max_by(array $values, callable $num_func) : ?int {
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

$c = max_by([1, 2, 3], function(int $a): int {
    return $a + mt_rand(0, $a);
});

echo $c;
===expect===
ArgumentTypeCoercion
===ignore===
TODO
