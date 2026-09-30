===description===
An integer `exit` operand is a status code, not output.
===config===
suppress=MixedArrayAccess
===file===
<?php
function test(): void {
    exit((int) $_GET['code']);
}
===expect===
