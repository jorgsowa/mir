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
//          ^^ PossiblyInvalidArgument: Argument $stream of fprintf() expects 'resource', possibly different type 'resource|false' provided
}

function outputStream(): void {
    $h = fopen('php://output', 'w');
    fprintf($h, "%s\n", $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
//          ^^ PossiblyInvalidArgument: Argument $stream of fprintf() expects 'resource', possibly different type 'resource|false' provided
}

function reassigned(): void {
    $h = fopen('/tmp/app.log', 'a');
    fclose($h);
//         ^^ PossiblyInvalidArgument: Argument $stream of fclose() expects 'resource', possibly different type 'resource|false' provided
    $h = fopen('php://stdout', 'w');
    fprintf($h, "%s\n", $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
//          ^^ PossiblyInvalidArgument: Argument $stream of fprintf() expects 'resource', possibly different type 'resource|false' provided
}

function onePath(bool $log): void {
    $h = fopen('php://output', 'w');
    if ($log) {
        $h = fopen('/tmp/app.log', 'a');
    }
    fprintf($h, "%s\n", $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
//          ^^ PossiblyInvalidArgument: Argument $stream of fprintf() expects 'resource', possibly different type 'resource|false' provided
}
