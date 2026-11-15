===description===
A key or IV written as a literal is shared by every deployment and message, and
decrypting an AEAD cipher without its tag always fails.
===file===
<?php
function crypt_it(string $data, string $key, string $iv): void {
    openssl_encrypt($data, 'aes-256-cbc', 'my-secret-key', 0, $iv);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WeakCryptography: Weak cryptography: the key is hardcoded
    openssl_encrypt($data, 'aes-256-cbc', $key, 0, '1234567890123456');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WeakCryptography: Weak cryptography: the IV is hardcoded; use a fresh random IV for every message
    openssl_decrypt($data, 'aes-256-gcm', $key, OPENSSL_RAW_DATA, $iv);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WeakCryptography: Weak cryptography: an AEAD cipher is decrypted without its authentication tag
}
