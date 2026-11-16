===description===
A suffix check without the leading dot accepts `eviltrusted.com`, and a check of
another variable doesn't cover the Origin; a `.trusted.com` suffix is a real check.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <TaintedHeader errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function suffixWithoutDot(): void {
    $origin = $_SERVER['HTTP_ORIGIN'];
    if (str_ends_with($origin, 'trusted.com')) {
        header("Access-Control-Allow-Origin: $origin");
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCors: CORS reflects the request's Origin while allowing credentials — any site can read authenticated responses
        header('Access-Control-Allow-Credentials: true');
    }
}

function otherVariableChecked(int $mode): void {
    $origin = $_SERVER['HTTP_ORIGIN'];
    if (in_array($mode, [1, 2], true)) {
        $mode = 0;
    }
    header("Access-Control-Allow-Origin: $origin");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCors: CORS reflects the request's Origin while allowing credentials — any site can read authenticated responses
    header('Access-Control-Allow-Credentials: true');
}

function subdomainSuffix(): void {
    $origin = $_SERVER['HTTP_ORIGIN'];
    if (str_ends_with($origin, '.trusted.com')) {
        header("Access-Control-Allow-Origin: $origin");
        header('Access-Control-Allow-Credentials: true');
    }
}
