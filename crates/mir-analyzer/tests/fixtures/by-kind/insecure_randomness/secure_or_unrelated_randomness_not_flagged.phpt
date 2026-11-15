===description===
Secure generators, weak randomness mixed with a secure part, weak randomness in
functions not named for a secret (or named for something about a secret), and a
variable reassigned with a secure value are fine.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function generateToken(): string {
    return bin2hex(random_bytes(32));
}

function generateCsrfToken(): string {
    return md5(uniqid('', true) . random_bytes(16));
}

function getTokenTtl(): int {
    return mt_rand(60, 120);
}

function pickColor(): string {
    $colors = ['red', 'green'];
    return $colors[array_rand($colors)];
}

function tokenize(string $s): string {
    return str_shuffle($s);
}

function generateNonce(): string {
    $n = uniqid();
    $n = bin2hex(random_bytes(16));
    return $n;
}

function requestId(): void {
    header('X-Request-Id: ' . uniqid());
    session_id(bin2hex(random_bytes(16)));
}

$makeToken = function (): string {
    return uniqid();
};
