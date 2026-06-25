===description===
G1: function with two template params — both are correctly inferred from the argument types
and substituted into the return type.
===config===
suppress=UnusedVariable,MissingReturnType,MissingPropertyType
===file===
<?php
/**
 * @template K
 * @template V
 * @param array<K, V> $arr
 * @return array<V, K>
 */
function flip_keys(array $arr): array {
    $result = [];
    foreach ($arr as $k => $v) {
        $result[$v] = $k;
    }
    return $result;
}

/** @var array<string, int> $map */
$map = ['a' => 1, 'b' => 2];
$flipped = flip_keys($map);
/** @mir-check $flipped is array<int, string> */
===expect===
