===description===
Tainted input in `setcookie()`/`setrawcookie()` reports TaintedCookie.
===config===
suppress=MixedArrayAccess,MixedArgument
===file===
<?php
function test(): void {
    setcookie('name', $_GET['v']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCookie: Tainted cookie — possible cookie injection
    setrawcookie('name', $_GET['v']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCookie: Tainted cookie — possible cookie injection
}
===expect===
