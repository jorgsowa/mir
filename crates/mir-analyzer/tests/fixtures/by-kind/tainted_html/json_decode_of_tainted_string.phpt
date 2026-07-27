===description===
json_decode() of a tainted string never propagated taint — attacker fully
controls the decoded structure's keys/values (a common route via
JSON-body web APIs), but the call result stayed untainted regardless of
its subject argument.
===config===
suppress=MixedArrayAccess,MixedArgument,MixedAssignment
===file===
<?php
function viaJsonDecode(): void {
    $data = json_decode($_GET['payload'], true);
    echo $data['name'];
}

function staticOnly(): void {
    $data = json_decode('{"name":"safe"}', true);
    echo $data['name'];
}
===expect===
TaintedHtml@4:4-4:23: Tainted HTML output — possible XSS
