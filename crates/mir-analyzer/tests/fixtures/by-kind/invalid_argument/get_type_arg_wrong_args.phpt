===description===
Get type arg wrong args
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//              ^^ InvalidArgument: Argument $var of testInt() expects 'int', got '"two"'

    case "integer":
        testString($a);
//                 ^^ PossiblyInvalidArgument: Argument $var of testString() expects 'string', possibly different type '1|"two"' provided
}
