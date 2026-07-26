===description===
A by-ref output parameter's written-back value derives from the call's
own arguments (preg_match's $matches derives from the tainted subject),
but the write-back site only ever called ctx.set_var for the new type —
never taint_var/clear_var_taint — so $matches stayed untainted even
though it holds pieces of a tainted string.
===config===
suppress=MixedArgument,MissingReturnType,MixedArrayAccess
php_version=8.2
===file===
<?php
function run(): void {
    preg_match('/(\d+)/', $_GET['input'], $matches);
    echo $matches[0];
}
===expect===
TaintedHtml@4:4-4:21: Tainted HTML output — possible XSS
