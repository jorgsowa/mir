===description===
A function or method named for a secret that returns a value from a guessable
generator, directly, hashed, built from random characters, or through a variable.
===file===
<?php
function generateResetToken(): string {
    return md5(uniqid());
//  ^^^^^^^^^^^^^^^^^^^^^ InsecureRandomness: Insecure randomness: the value generateResetToken() returns comes from a non-cryptographic generator; use random_bytes() or random_int()
}

function make_salt(): string {
    $salt = substr(str_shuffle('abcdefghijklmnopqrstuvwxyz0123456789'), 0, 16);
    return $salt;
//  ^^^^^^^^^^^^^ InsecureRandomness: Insecure randomness: the value make_salt() returns comes from a non-cryptographic generator; use random_bytes() or random_int()
}

final class Codes {
    public function newPasswordResetCode(): int {
        return mt_rand(100000, 999999);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureRandomness: Insecure randomness: the value newPasswordResetCode() returns comes from a non-cryptographic generator; use random_bytes() or random_int()
    }

    public function createApiKey(): string {
        $chars = 'abcdef0123456789';
        return 'key_' . $chars[rand(0, 15)] . sha1((string) lcg_value());
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureRandomness: Insecure randomness: the value createApiKey() returns comes from a non-cryptographic generator; use random_bytes() or random_int()
    }
}
