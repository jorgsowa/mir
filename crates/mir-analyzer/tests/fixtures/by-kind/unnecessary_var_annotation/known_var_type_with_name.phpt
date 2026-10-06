===description===
Known var type with name
===file===
<?php
function foo() : string {
    return "hello";
}

/** @var string $a */
$a = foo();
//<^^^^^^^^^^^ UnnecessaryVarAnnotation: @var annotation for $a is unnecessary

echo $a;
