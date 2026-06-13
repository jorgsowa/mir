===description===
Match true impossible
===config===
suppress=UnusedVariable
===file===
<?php
$foo = new stdClass();
$a = match (true) {
    $foo instanceof stdClass => 1,
    $foo instanceof Exception => 1,
};
===expect===
