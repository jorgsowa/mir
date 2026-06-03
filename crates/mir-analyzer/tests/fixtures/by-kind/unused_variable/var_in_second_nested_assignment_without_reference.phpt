===description===
Var in second nested assignment without reference
===file===
<?php
if (rand(0, 1)) {
    $a = "foo";
    echo $a;
}

if (rand(0, 1)) {
    $a = "foo";
}
===expect===
UnusedVariable@8:5-8:7: Variable $a is never read
