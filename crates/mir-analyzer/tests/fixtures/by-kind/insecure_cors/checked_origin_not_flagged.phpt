===description===
An origin checked against an allowlist or by a validating call before it's echoed
back, and a wildcard origin, which browsers refuse with credentials, aren't
reported.
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
function allowedOrigin(string $origin): bool {
    return str_ends_with($origin, '.example.com');
}

/** @param list<string> $allowed */
function api(array $allowed): void {
    $origin = $_SERVER['HTTP_ORIGIN'] ?? '';
    if (in_array($origin, $allowed, true)) {
        header('Access-Control-Allow-Origin: ' . $origin);
        header('Access-Control-Allow-Credentials: true');
    }
}

function viaHelper(): void {
    if (!allowedOrigin((string) $_SERVER['HTTP_ORIGIN'])) {
        return;
    }
    header('Access-Control-Allow-Origin: ' . $_SERVER['HTTP_ORIGIN']);
    header('Access-Control-Allow-Credentials: true');
}

function wildcard(): void {
    header('Access-Control-Allow-Origin: *');
}
