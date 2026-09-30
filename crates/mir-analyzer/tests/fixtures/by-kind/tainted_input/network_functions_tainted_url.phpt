===description===
Functions that open a URL or host from their first argument are SSRF sinks.
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
function test(): void {
    curl_init($_GET['u']);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    get_headers($_GET['u']);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    fsockopen($_GET['h'], 80);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
}
===expect===
