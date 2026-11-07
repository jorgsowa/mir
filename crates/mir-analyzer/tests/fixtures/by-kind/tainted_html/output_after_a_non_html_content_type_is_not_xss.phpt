===description===
Output after a literal non-HTML `Content-Type` header on every path isn't HTML. A later HTML or
unknown content type, or a header sent on one path only, keeps it reported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function json(): void {
    header('Content-Type: application/json; charset=utf-8');
    echo json_encode($_GET);
}

function plain(string $charset): void {
    header('content-type: text/plain; charset=' . $charset);
    echo $_GET['x'];
}

function sendJson(array $data): void {
    header('Content-Type: application/json');
    echo json_encode($data);
}

function html(): void {
    header('Content-Type: application/json');
    header('Content-Type: text/html');
    echo $_GET['x'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function unknown(string $h): void {
    header('Content-Type: application/json');
    header($h);
    echo $_GET['x'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function onePath(bool $api): void {
    if ($api) {
        header('Content-Type: application/json');
    }
    echo $_GET['x'];
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function exitingPath(bool $api): void {
    if (!$api) {
        echo 'page';
        exit;
    }
    header('Content-Type: application/json');
    echo $_GET['x'];
}

sendJson($_GET);
