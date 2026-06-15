===description===
Var defined in if without reference
===file===
<?php
$a = 5;
if (rand(0, 1)) {
    $b = "hello";
} else {
    $b = "goodbye";
}
===expect===
UnusedVariable@2:0-2:2: Variable $a is never read
UnusedVariable@4:4-4:6: Variable $b is never read
