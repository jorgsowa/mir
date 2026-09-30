===description===
`setcookie()` is not an HTML output sink.
===config===
suppress=MixedArrayAccess,MixedArgument
===file===
<?php
function test(): void {
    setcookie('name', $_GET['v']);
}
===expect===
