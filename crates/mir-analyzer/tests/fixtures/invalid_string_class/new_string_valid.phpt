===description===
new with string variable should not error
===file===
<?php
function test(string $className) {
    new $className();
}
===expect===
