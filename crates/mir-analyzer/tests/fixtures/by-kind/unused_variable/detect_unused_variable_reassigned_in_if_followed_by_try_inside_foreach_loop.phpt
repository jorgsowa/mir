===description===
(divergence from Psalm: $user = null is READ by the later
`$user !== null` check on the not-reassigned path, so it is not reported)
Detect unused variable reassigned in if followed by try inside foreach loop
===file===
<?php
$user_id = 0;
$user = null;

if (rand(0, 1)) {
    $user_id = rand(0, 1);
    $user = $user_id;
}

if ($user !== null && $user !== 0) {
    $a = 0;
    foreach ([1, 2, 3] as $i) {
        $a += $i;
        try {} catch (Exception $e) {}
    }
    echo $i;
}
===expect===
UnusedVariable@2:0-2:8: Variable $user_id is never read
UnusedVariable@13:8-13:10: Variable $a is never read
UnusedVariable@14:21-14:38: Variable $e is never read
PossiblyUndefinedVariable@16:9-16:11: Variable $i might not be defined
