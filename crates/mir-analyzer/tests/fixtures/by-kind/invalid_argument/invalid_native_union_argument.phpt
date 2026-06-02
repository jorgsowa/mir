===description===
Invalid native union argument
===file===
<?php
function test(string|null $in): string|null {
    return $in;
}
test(2);

===expect===
InvalidArgument@5:6-5:7: Argument $in of test() expects 'string|null', got '2'
