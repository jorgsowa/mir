===description===
`fprintf` to a handle `fopen()` opened on a literal file path is no HTML output; an output stream,
a reassigned handle or one opened on a single path still counts.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function logFile(): void {
    $h = fopen('/tmp/app.log', 'a');
    fprintf($h, "%s\n", $_GET['x']);
}

function outputStream(): void {
    $h = fopen('php://output', 'w');
    fprintf($h, "%s\n", $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function reassigned(): void {
    $h = fopen('/tmp/app.log', 'a');
    fclose($h);
    $h = fopen('php://stdout', 'w');
    fprintf($h, "%s\n", $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function onePath(bool $log): void {
    $h = fopen('php://output', 'w');
    if ($log) {
        $h = fopen('/tmp/app.log', 'a');
    }
    fprintf($h, "%s\n", $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
