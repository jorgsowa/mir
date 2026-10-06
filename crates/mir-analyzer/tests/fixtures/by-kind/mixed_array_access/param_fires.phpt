===description===
MixedArrayAccess fires when indexing into a mixed-typed parameter.
===file===
<?php
function foo(mixed $a): void {
    echo $a[0];
//       ^^^^^ MixedArrayAccess: Array access on mixed type
}
