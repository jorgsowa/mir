===description===
Get type arg wrong args
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
InvalidArgument@14:17-14:19: Argument $var of testInt() expects 'int', got '1|"two"'
InvalidArgument@17:20-17:22: Argument $var of testString() expects 'string', got '1|"two"'
