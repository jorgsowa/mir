===description===
Known var type
===file===
<?php
function foo() : string {
    return "hello";
}

/** @var string */
$a = foo();

echo $a;
===expect===
UnnecessaryVarAnnotation@7:0-7:11: @var annotation for $a is unnecessary
