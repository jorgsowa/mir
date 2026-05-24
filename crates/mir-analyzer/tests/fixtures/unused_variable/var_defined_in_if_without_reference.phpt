===description===
varDefinedInIfWithoutReference
===file===
<?php
$a = 5;
if (rand(0, 1)) {
    $b = "hello";
} else {
    $b = "goodbye";
}
===expect===
UnusedVariable
===ignore===
TODO
