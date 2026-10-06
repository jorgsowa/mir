===description===
Known var type
===file===
<?php
function foo() : string {
    return "hello";
}

/** @var string */
$a = foo();
//<^^^^^^^^^^^ UnnecessaryVarAnnotation: @var annotation for $a is unnecessary

echo $a;
