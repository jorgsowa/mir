===description===
Known var type
===ignore===
TODO
===file===
<?php
function foo() : string {
    return "hello";
}

/** @var string */
$a = foo();

echo $a;
===expect===
