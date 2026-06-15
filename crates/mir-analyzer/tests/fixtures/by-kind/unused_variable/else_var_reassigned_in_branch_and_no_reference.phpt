===description===
Else var reassigned in branch and no reference
===file===
<?php
$a = true;

if (rand(0, 1)) {
    // do nothing
} else {
    $a = false;
}
===expect===
UnusedVariable@2:0-2:2: Variable $a is never read
