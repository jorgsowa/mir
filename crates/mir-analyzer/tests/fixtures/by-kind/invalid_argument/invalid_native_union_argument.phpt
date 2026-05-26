===description===
Invalid native union argument
===file===
<?php
function test(string|null $in): string|null {
    return $in;
}
test(2);

===expect===
InvalidScalarArgument
===ignore===
TODO
