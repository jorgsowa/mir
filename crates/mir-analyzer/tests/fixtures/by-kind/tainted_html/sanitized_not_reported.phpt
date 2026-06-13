===description===
sanitized not reported
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
function test(): void {
    echo htmlspecialchars($_GET['x']);
}
===expect===
