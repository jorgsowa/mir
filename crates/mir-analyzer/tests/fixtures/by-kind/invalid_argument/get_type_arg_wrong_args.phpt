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
//              ^^ PossiblyInvalidArgument: Argument $var of testInt() expects 'int', possibly different type '1|"two"' provided

    case "integer":
        testString($a);
//                 ^^ PossiblyInvalidArgument: Argument $var of testString() expects 'string', possibly different type '1|"two"' provided
}
===expect===
