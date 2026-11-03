===description===
A by-reference closure capture can gain keys through the shared reference before the closure runs, so its shape is open inside the closure.
===file===
<?php
function fill(array &$info): void {
    $info['request_header'] = 'GET / HTTP/1.1';
}

function run(): void {
    $info = ['debug' => '', 'total_time' => 0.0];
    $notify = static function (int $code) use (&$info): void {
        if (2 === $code) {
            $info['debug'] .= $info['request_header'];
            unset($info['request_header']);
        }
    };
    fill($info);
    $notify(2);
}
