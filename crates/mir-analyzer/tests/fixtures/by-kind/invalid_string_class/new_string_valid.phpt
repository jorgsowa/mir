===description===
new with string variable should not error
===config===
suppress=MissingReturnType
===file===
<?php
function test(string $className) {
    new $className();
}
===expect===
