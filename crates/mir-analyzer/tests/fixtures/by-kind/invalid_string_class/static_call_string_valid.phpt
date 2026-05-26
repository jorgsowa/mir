===description===
static call with string variable should not error
===file===
<?php
function test(string $className) {
    $className::method();
}
===expect===
