===description===
Get type arg wrong args
===config===
suppress=UnusedParam
===file===
<?php
function testInt(int $var): void {

}

function testString(string $var): void {

}

$a = rand(0, 10) ? 1 : "two";

switch (gettype($a)) {
    case "string":
        testInt($a);

    case "integer":
        testString($a);
}
===expect===
PossiblyInvalidArgument@14:16-14:18: Argument $var of testInt() expects 'int', possibly different type '1|"two"' provided
PossiblyInvalidArgument@17:19-17:21: Argument $var of testString() expects 'string', possibly different type '1|"two"' provided
