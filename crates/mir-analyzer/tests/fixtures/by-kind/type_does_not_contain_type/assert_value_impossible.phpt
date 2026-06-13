===description===
Assert value impossible
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @assert "foo"|"bar"|"foo-bar" $s
 */
function assertFooBar(string $s) : void {
}

$a = "";
assertFooBar($a);
===expect===
