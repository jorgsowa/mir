===description===
unusedIfVarInBranch
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
UnusedVariable
===ignore===
TODO
