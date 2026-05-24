===description===
unusedVarWithConditionalIncrement
===file===
<?php
$a = 5;
if (rand(0, 1)) {
    $a++;
}
===expect===
UnusedVariable
===ignore===
TODO
