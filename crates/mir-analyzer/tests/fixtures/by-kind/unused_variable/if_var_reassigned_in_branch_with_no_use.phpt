===description===
If var reassigned in branch with no use
===file===
<?php
$a = true;

if (rand(0, 1)) {
    $a = false;
}
===expect===
UnusedVariable@2:0-2:2: Variable $a is never read
