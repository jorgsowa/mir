===description===
$obj::method() where $obj is plain object type should not error
===file===
<?php
function test(object $obj): void {
    $obj::bar();
}
===expect===
