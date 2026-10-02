===description===
Non-literal string callee cannot premark
===file===
<?php
function e(string $fn): void {
    $fn('/a/', 'b', $m);
//                  ^^ UndefinedVariable: Variable $m is not defined
}
===expect===
