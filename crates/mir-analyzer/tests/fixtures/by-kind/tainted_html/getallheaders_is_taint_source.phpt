===description===
getallheaders()/apache_request_headers() return raw HTTP request headers
-- as attacker-controlled as any superglobal -- but neither was ever
treated as a taint source.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <PossiblyInvalidArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fromGetAllHeaders(): void {
    $headers = getallheaders();
    echo $headers['User-Agent'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function fromApacheRequestHeaders(): void {
    $headers = apache_request_headers();
    echo $headers['User-Agent'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
