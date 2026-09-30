===description===
`vfprintf()` writes formatted output like `fprintf()`.
===config===
suppress=MixedArgument,MixedArrayAccess,MissingParamType
===file===
<?php
function test($out): void {
    vfprintf($out, '%s', [$_GET['x']]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
