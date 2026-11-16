===description===
The credentials header counts whether it's sent before or after the reflected
origin, in any case or spacing; one that's only in a comment doesn't.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <TaintedHeader errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function credentialsFirst(): void {
    header('Access-Control-Allow-Credentials:true');
    header('Access-Control-Allow-Origin: ' . $_SERVER['HTTP_ORIGIN']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCors: CORS reflects the request's Origin while allowing credentials — any site can read authenticated responses
}

function credentialsAfter(): void {
    header('Access-Control-Allow-Origin: ' . $_SERVER['HTTP_ORIGIN']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCors: CORS reflects the request's Origin while allowing credentials — any site can read authenticated responses
    header('Access-Control-Allow-Credentials: TRUE');
}

function credentialsOnlyInComment(): void {
    // header('Access-Control-Allow-Credentials: true');
    header('Access-Control-Allow-Origin: ' . $_SERVER['HTTP_ORIGIN']);
}
