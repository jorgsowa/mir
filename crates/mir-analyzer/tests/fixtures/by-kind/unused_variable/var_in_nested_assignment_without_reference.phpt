===description===
Var in nested assignment without reference
===file===
<?php
if (rand(0, 1)) {
    $a = "foo";
}
===expect===
UnusedVariable
===ignore===
TODO
