===description===
Non-literal string callee cannot premark
===file===
<?php
function e(string $fn): void {
    $fn('/a/', 'b', $m);
}
===expect===
UndefinedVariable@3:20-3:22: Variable $m is not defined
