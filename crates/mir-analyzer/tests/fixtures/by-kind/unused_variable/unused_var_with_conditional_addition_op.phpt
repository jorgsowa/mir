===description===
Unused var with conditional addition op
===file===
<?php
$a = 5;
if (rand(0, 1)) {
    $a += 1;
}
===expect===
UnusedVariable
===ignore===
TODO
