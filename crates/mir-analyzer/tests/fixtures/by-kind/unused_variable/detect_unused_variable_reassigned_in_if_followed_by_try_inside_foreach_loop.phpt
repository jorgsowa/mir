===description===
(divergence from Psalm: $user = null is READ by the later
`$user !== null` check on the not-reassigned path, so it is not reported)
Detect unused variable reassigned in if followed by try inside foreach loop
===file===
<?php
$user_id = 0;
//<^^^^^^^^ UnusedVariable: Variable $user_id is never read
$user = null;

if (rand(0, 1)) {
    $user_id = rand(0, 1);
    $user = $user_id;
}

if ($user !== null && $user !== 0) {
    $a = 0;
    foreach ([1, 2, 3] as $i) {
        $a += $i;
//      ^^ UnusedVariable: Variable $a is never read
        try {} catch (Exception $e) {}
    }
    echo $i;
}
===expect===
