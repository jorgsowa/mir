===description===
A token appended one weak random character at a time stays weak; the same loop
over `random_int()` is fine.
===file===
<?php
function makeNonce(): string {
    $chars = 'abcdef0123456789';
    $token = '';
    for ($i = 0; $i < 32; $i++) {
        $token .= $chars[mt_rand(0, 15)];
    }
    return $token;
//  ^^^^^^^^^^^^^^ InsecureRandomness: Insecure randomness: the value makeNonce() returns comes from a non-cryptographic generator; use random_bytes() or random_int()
}

function makeSecureNonce(): string {
    $chars = 'abcdef0123456789';
    $token = '';
    for ($i = 0; $i < 32; $i++) {
        $token .= $chars[random_int(0, 15)];
    }
    return $token;
}
