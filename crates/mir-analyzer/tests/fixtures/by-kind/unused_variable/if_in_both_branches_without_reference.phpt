===description===
If in both branches without reference
===file===
<?php
$a = 5;
if (rand(0, 1)) {
    $b = "hello";
//  ^^ UnusedVariable: Variable $b is never read
} else {
    $b = "goodbye";
}
echo $a;
