===description===
A by-ref param in a bundled stub with a default or without a native type is an
out-param: PHP's arginfo leaves it untyped and never reads the incoming value,
so a variable that still holds something else can be passed (`preg_match($p,
$s, $m)` with a string `$m`, `openssl_sign($d, $sig, $k)` with a null `$sig`).
The declared type is what the call writes back. In/out params that read the
incoming value (the intl `$offset`, `array_multisort`, `mysqli::poll`) keep
their incoming type check.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function reuse(string $subject): int {
    $matches = 'previous';
    preg_match('/(a)/', $subject, $matches);
    /** @mir-check $matches is list<string> */
    $all = 5;
    preg_match_all('/a/', $subject, $all);
    $count = 'none';
    str_replace('a', 'b', $subject, $count);
    /** @mir-check $count is int */
    $percent = 'n/a';
    similar_text('a', 'b', $percent);
    /** @mir-check $percent is float */
    return $count;
}

function offset(NumberFormatter $fmt): void {
    $pos = 'start';
    $fmt->parse('12', NumberFormatter::TYPE_DOUBLE, $pos);
//                                                  ^^^^ InvalidArgument: Argument $offset of parse() expects 'int|null', got '"start"'
}

function null_initialized(OpenSSLAsymmetricKey $key, Socket $socket): string {
    $signature = null;
    openssl_sign('data', $signature, $key);
    /** @mir-check $signature is string */
    $pipes = null;
    proc_open('ls', [], $pipes);
    $status = null;
    pcntl_waitpid(1, $status);
    /** @mir-check $status is int */
    $hosts = null;
    getmxrr('example.com', $hosts);
    $buffer = null;
    socket_recv($socket, $buffer, 1, 0);
    return $signature . count($pipes) . count($hosts) . $status . $buffer;
}

function in_out_params(): void {
    $sorted = 'abc';
    array_multisort($sorted);
//                  ^^^^^^^ InvalidArgument: Argument $array of array_multisort() expects 'array', got '"abc"'
    $read = 'conn';
    $error = [];
    $reject = [];
    mysqli::poll($read, $error, $reject, 1);
//               ^^^^^ InvalidArgument: Argument $read of poll() expects 'array|null', got '"conn"'
}
