===description===
compact('id') copies $id's current value into the returned array under
key 'id', but the taint check never consulted the named variable's taint
state — echoing the result silently produced no diagnostic even when the
source variable was tainted.
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
function viaCompact(): void {
    $id = $_GET['id'];
    $data = compact('id');
    echo $data['id'];
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function safeCompactOnly(): void {
    $id = 5;
    $data = compact('id');
    echo $data['id'];
}
===expect===
