===description===
Invalid native union argument
===file===
<?php
function test(string|null $in): string|null {
    return $in;
}
test(2);

===expect===
ArgumentTypeCoercion@5:5-5:6: Argument $in of test() expects 'string|null', got '2' — coercion may fail at runtime
