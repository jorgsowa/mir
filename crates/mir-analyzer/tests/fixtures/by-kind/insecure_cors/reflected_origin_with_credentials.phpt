===description===
Echoing the request's `Origin` back with credentials allowed lets any site read
responses made with the user's cookies.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <TaintedHeader errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function api(): void {
    header('Access-Control-Allow-Origin: ' . $_SERVER['HTTP_ORIGIN']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCors: CORS reflects the request's Origin while allowing credentials — any site can read authenticated responses
    header('Access-Control-Allow-Credentials: true');
}

function presenceOnly(): void {
    if (isset($_SERVER['HTTP_ORIGIN'])) {
        $origin = $_SERVER['HTTP_ORIGIN'];
        header("Access-Control-Allow-Origin: $origin");
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCors: CORS reflects the request's Origin while allowing credentials — any site can read authenticated responses
        header('Access-Control-Allow-Credentials: true');
    }
}
