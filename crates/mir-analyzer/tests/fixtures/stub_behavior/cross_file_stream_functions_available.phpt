===description===
cross file stream functions available
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:StreamHelper.php===
<?php
function isTerminalStream(mixed $stream): bool {
    return stream_isatty($stream);
//                       ^^^^^^^ MixedArgument: Argument $stream of stream_isatty() is mixed
}

function copyStream(mixed $from, mixed $to): int|false {
    return stream_copy_to_stream($from, $to);
//                               ^^^^^ MixedArgument: Argument $from of stream_copy_to_stream() is mixed
//                                      ^^^ MixedArgument: Argument $to of stream_copy_to_stream() is mixed
}
===file:Main.php===
<?php
$stdin = fopen('php://stdin', 'r');
if ($stdin !== false) {
    $isTty = isTerminalStream($stdin);
}
