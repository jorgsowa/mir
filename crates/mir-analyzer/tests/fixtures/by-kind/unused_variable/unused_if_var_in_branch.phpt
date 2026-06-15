===description===
Unused if var in branch
===file===
<?php
if (rand(0, 1)) {

} elseif (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = "foo";
    } else {
        $a = "bar";
        echo $a;
    }
}
===expect===
UnusedVariable@6:8-6:10: Variable $a is never read
