===description===
Assert value impossible
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
