===description===
If in both branches without reference
===file===
<?php
$a = 5;
if (rand(0, 1)) {
    $b = "hello";
} else {
    $b = "goodbye";
}
echo $a;
===expect===
UnusedVariable
===ignore===
TODO
