===description===
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
PossiblyUndefinedVariable@16:10-16:12: Variable $i might not be defined
