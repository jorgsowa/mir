===description===
Known var type with name
===file===
<?php
function foo() : string {
    return "hello";
}

/** @var string $a */
$a = foo();

echo $a;
===expect===
UnnecessaryVarAnnotation
===ignore===
TODO
