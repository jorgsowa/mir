===description===
Known var type with name
===ignore===
TODO
===file===
<?php
function foo() : string {
    return "hello";
}

/** @var string $a */
$a = foo();

echo $a;
===expect===
