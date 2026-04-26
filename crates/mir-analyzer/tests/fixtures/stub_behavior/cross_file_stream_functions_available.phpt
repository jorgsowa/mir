===file:StreamHelper.php===
<?php
function isTerminalStream(mixed $stream): bool {
    return stream_isatty($stream);
}

function copyStream(mixed $from, mixed $to): int|false {
    return stream_copy_to_stream($from, $to);
}
===file:Main.php===
<?php
$stdin = fopen('php://stdin', 'r');
if ($stdin !== false) {
    $isTty = isTerminalStream($stdin);
}
===expect===
