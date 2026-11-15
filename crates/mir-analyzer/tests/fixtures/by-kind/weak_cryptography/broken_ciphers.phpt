===description===
ECB mode and the DES, RC2, RC4 and Blowfish cipher families are broken.
===file===
<?php
function encrypt(string $data, string $key, string $iv): void {
    openssl_encrypt($data, 'aes-256-ecb', $key);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WeakCryptography: Weak cryptography: ECB mode of 'aes-256-ecb' leaks patterns of the plaintext
    openssl_encrypt($data, 'DES-EDE3-CBC', $key, 0, $iv);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WeakCryptography: Weak cryptography: cipher 'des-ede3-cbc' is broken
    openssl_decrypt($data, 'rc4', $key);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WeakCryptography: Weak cryptography: cipher 'rc4' is broken
    openssl_encrypt($data, 'bf-cbc', $key, 0, $iv);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WeakCryptography: Weak cryptography: cipher 'bf-cbc' is broken
}
