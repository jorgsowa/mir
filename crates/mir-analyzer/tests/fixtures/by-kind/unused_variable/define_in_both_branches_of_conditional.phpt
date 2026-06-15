===description===
Define in both branches of conditional
===file===
<?php
$i = null;

if (($i = rand(0, 5)) || ($i = rand(0, 3))) {
    echo $i;
}
===expect===
UnusedVariable@2:0-2:2: Variable $i is never read
