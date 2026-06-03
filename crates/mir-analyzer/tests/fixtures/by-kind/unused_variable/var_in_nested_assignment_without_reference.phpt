===description===
Var in nested assignment without reference
===file===
<?php
if (rand(0, 1)) {
    $a = "foo";
}
===expect===
UnusedVariable@3:5-3:7: Variable $a is never read
