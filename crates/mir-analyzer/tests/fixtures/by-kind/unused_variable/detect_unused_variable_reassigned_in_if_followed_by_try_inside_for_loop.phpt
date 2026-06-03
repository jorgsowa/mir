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
