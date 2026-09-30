===description===
Negative control for the imap stub vendoring fix (Sector I1): vendoring the
real stub file must not turn `imap_*` into a wildcard-resolved namespace —
a function that isn't actually part of the extension must still be
flagged undefined.
===file===
<?php
function f(): void {
    imap_open_not_a_real_function('x', 'y', 'z');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function imap_open_not_a_real_function() is not defined
}
===expect===
