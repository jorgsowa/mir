===description===
Var in nested assignment without reference
===file===
<?php
if (rand(0, 1)) {
    $a = "foo";
}
===expect===
UnusedVariable@3:4-3:6: Variable $a is never read
