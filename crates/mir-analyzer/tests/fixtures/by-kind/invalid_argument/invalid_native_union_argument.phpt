===description===
Invalid native union argument
===file===
<?php
function test(string|null $in): string|null {
    return $in;
}
test(2);
//   ^ ArgumentTypeCoercion: Argument $in of test() expects 'string|null', got '2' — coercion may fail at runtime

===expect===
