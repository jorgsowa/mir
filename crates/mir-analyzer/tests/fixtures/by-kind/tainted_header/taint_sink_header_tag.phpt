===description===
`@taint-sink header $value` raises TaintedHeader.
===config===
suppress=MixedArrayAccess,MixedArgument,UnusedParam
===file===
<?php
/** @taint-sink header $value */
function send_header(string $value): void {}

function test(): void {
    send_header($_GET['v']);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
}
===expect===
