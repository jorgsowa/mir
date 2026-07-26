===description===
A tainted value returned from an immediately-invoked, argument-less arrow
function ((fn() => $_GET['x'])()) wasn't recognized as tainted at all —
narrower than the general call-result taint pass-through this module
deliberately doesn't model, since an arrow function's single-expression
body can be checked directly against the same ctx with no params to
shadow the outer scope.
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
function test(): void {
    echo (fn() => $_GET['x'])();
}
===expect===
TaintedHtml@3:4-3:32: Tainted HTML output — possible XSS
