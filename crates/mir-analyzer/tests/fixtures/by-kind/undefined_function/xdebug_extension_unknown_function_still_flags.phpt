===description===
Negative control for the xdebug stub vendoring fix (Sector I1): vendoring
the real stub file must not turn `xdebug_*` into a wildcard-resolved
namespace — a function that isn't actually part of the extension must
still be flagged undefined.
===file===
<?php
function f(): void {
    xdebug_break_not_a_real_function();
}
===expect===
UndefinedFunction@3:4-3:38: Function xdebug_break_not_a_real_function() is not defined
