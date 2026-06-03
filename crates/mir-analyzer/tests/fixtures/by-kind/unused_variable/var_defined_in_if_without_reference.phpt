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
UnusedVariable@2:1-2:3: Variable $a is never read
UnusedVariable@4:5-4:7: Variable $b is never read
