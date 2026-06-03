===description===
Detect unused variable reassigned in if followed by try inside for loop
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
    for ($i = 1; $i <= 10; $i++) {
        $a += $i;
        try {} catch (Exception $e) {}
    }
    echo $i;
}
===expect===
UnusedVariable@2:1-2:9: Variable $user_id is never read
UnusedVariable@3:1-3:6: Variable $user is never read
UnusedVariable@13:9-13:11: Variable $a is never read
UnusedVariable@14:22-14:39: Variable $e is never read
