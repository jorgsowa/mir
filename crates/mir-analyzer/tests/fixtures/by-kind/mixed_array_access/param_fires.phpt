===description===
MixedArrayAccess fires when indexing into a mixed-typed parameter.
===file===
<?php
function foo(mixed $a): void {
    echo $a[0];
}
===expect===
MixedArrayAccess@3:9-3:14: Array access on mixed type
