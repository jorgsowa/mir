===description===
knownVarType
===file===
<?php
function foo() : string {
    return "hello";
}

/** @var string */
$a = foo();

echo $a;
===expect===
UnnecessaryVarAnnotation
===ignore===
TODO
