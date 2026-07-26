===description===
A superglobal passed by reference to a builtin (sort($_SESSION)) or
unset() on a superglobal array element (unset($_SESSION['key'])) mutates
external, shared state exactly as much as a plain assignment would — both
previously bypassed ImpureGlobalVariable entirely, since check_byref_arg_purity
and the unset() array-index unwrap loop had no bare-Variable arm at all.
Fixed as a side effect of the shared check_var_write_purity helper added
for the by-ref-parameter sector.
===config===
suppress=MixedArgument,MixedArrayAccess,ImpureFunctionCall
===file===
<?php
/** @pure */
function f(): void {
    sort($_SESSION);
    unset($_SESSION['key']);
}
===expect===
ImpureGlobalVariable@4:9-4:18: Using global variable $_SESSION in a @pure function
ImpureGlobalVariable@5:10-5:26: Using global variable $_SESSION in a @pure function
