===description===
compact('id') copies $id's current value into the returned array under
key 'id', but the taint check never consulted the named variable's taint
state — echoing the result silently produced no diagnostic even when the
source variable was tainted.
===config===
suppress=MixedArrayAccess,MixedArgument,MixedAssignment
===file===
<?php
function viaCompact(): void {
    $id = $_GET['id'];
    $data = compact('id');
    echo $data['id'];
}

function safeCompactOnly(): void {
    $id = 5;
    $data = compact('id');
    echo $data['id'];
}
===expect===
TaintedHtml@5:4-5:21: Tainted HTML output — possible XSS
