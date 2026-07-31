===description===
Negative control for the ssh2 stub vendoring fix (Sector I1): vendoring the
real stub file must not turn `ssh2_*` into a wildcard-resolved namespace —
a function that isn't actually part of the extension must still be
flagged undefined.
===file===
<?php
function f(): void {
    ssh2_connect_not_a_real_function('x');
}
===expect===
UndefinedFunction@3:4-3:41: Function ssh2_connect_not_a_real_function() is not defined
