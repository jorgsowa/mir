===description===
json_decode() of a tainted string never propagated taint — attacker fully
controls the decoded structure's keys/values (a common route via
JSON-body web APIs), but the call result stayed untainted regardless of
its subject argument.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function viaJsonDecode(): void {
    $data = json_decode($_GET['payload'], true);
    echo $data['name'];
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function staticOnly(): void {
    $data = json_decode('{"name":"safe"}', true);
    echo $data['name'];
}
