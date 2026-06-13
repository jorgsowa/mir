===description===
static call with string variable should not error
===config===
suppress=MissingReturnType
===file===
<?php
function test(string $className) {
    $className::method();
}
===expect===
