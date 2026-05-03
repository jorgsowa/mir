===description===
match arm not reported
===file===
<?php
function test(string $type): string {
    $msg = 'hello';
    return match($type) {
        'upper' => strtoupper($msg),
        default => $msg,
    };
}
===expect===
===ignore===
TODO
