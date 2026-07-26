===description===
getallheaders()/apache_request_headers() return raw HTTP request headers
-- as attacker-controlled as any superglobal -- but neither was ever
treated as a taint source.
===config===
suppress=MixedArgument,MixedArrayAccess,PossiblyInvalidArrayAccess
===file===
<?php
function fromGetAllHeaders(): void {
    $headers = getallheaders();
    echo $headers['User-Agent'];
}

function fromApacheRequestHeaders(): void {
    $headers = apache_request_headers();
    echo $headers['User-Agent'];
}
===expect===
TaintedHtml@4:4-4:32: Tainted HTML output — possible XSS
TaintedHtml@9:4-9:32: Tainted HTML output — possible XSS
