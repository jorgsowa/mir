===description===
Each `if` that optionally sets a key doubles the shapes in a union; past a small limit they fold into shapes with optional keys instead of growing 2^n (which made loop widening quadratic). Below the limit the shapes stay apart.
===file===
<?php
function few_branches_stay_apart(bool $a): void {
    $p = ['key' => 1];
    if ($a) {
        $p['subject'] = 's';
    }
    /** @mir-check $p is array{'key': 1, 'subject': 's'}|array{'key': 1} */
    echo json_encode($p);
}

/** @param list<string> $xs */
function many_branches_then_loop(array $xs, bool $c): int {
    $p = ['m' => ['a' => 1]];
    if ($c) { $p['m']['k1'] = 1; }
    if ($c) { $p['m']['k2'] = 1; }
    if ($c) { $p['m']['k3'] = 1; }
    if ($c) { $p['m']['k4'] = 1; }
    if ($c) { $p['m']['k5'] = 1; }
    if ($c) { $p['m']['k6'] = 1; }
    if ($c) { $p['m']['k7'] = 1; }
    if ($c) { $p['m']['k8'] = 1; }
    if ($c) { $p['m']['k9'] = 1; }
    if ($c) { $p['m']['k10'] = 1; }
    if ($c) { $p['m']['k11'] = 1; }
    if ($c) { $p['m']['k12'] = 1; }
    if ($c) { $p['m']['k13'] = 1; }
    if ($c) { $p['m']['k14'] = 1; }
    if ($c) { $p['m']['k15'] = 1; }
    if ($c) { $p['m']['k16'] = 1; }
    foreach ($xs as $x) {
        $p['m']['h'][] = $x;
    }
    $a = $p['m']['a'];
    /** @mir-check $a is 1 */
    return $a;
}

/** @param list<string> $xs */
function two_keys_per_branch(array $xs, bool $c): void {
    $p = ['a' => 1];
    if ($c) { $p['x1'] = 1; $p['y1'] = 1; }
    if ($c) { $p['x2'] = 1; $p['y2'] = 1; }
    if ($c) { $p['x3'] = 1; $p['y3'] = 1; }
    if ($c) { $p['x4'] = 1; $p['y4'] = 1; }
    if ($c) { $p['x5'] = 1; $p['y5'] = 1; }
    if ($c) { $p['x6'] = 1; $p['y6'] = 1; }
    if ($c) { $p['x7'] = 1; $p['y7'] = 1; }
    if ($c) { $p['x8'] = 1; $p['y8'] = 1; }
    if ($c) { $p['x9'] = 1; $p['y9'] = 1; }
    if ($c) { $p['x10'] = 1; $p['y10'] = 1; }
    if ($c) { $p['x11'] = 1; $p['y11'] = 1; }
    if ($c) { $p['x12'] = 1; $p['y12'] = 1; }
    if ($c) { $p['x13'] = 1; $p['y13'] = 1; }
    if ($c) { $p['x14'] = 1; $p['y14'] = 1; }
    if ($c) { $p['x15'] = 1; $p['y15'] = 1; }
    if ($c) { $p['x16'] = 1; $p['y16'] = 1; }
    foreach ($xs as $x) {
        $p['h'][] = $x;
    }
    echo json_encode($p);
}
