===description===
AEAD encryption with a random IV, decryption with the tag, CBC with a runtime key
and IV, and ciphers chosen at runtime are fine.
===file===
<?php
function ok(string $data, string $key, string $cipher, string $tag): string {
    $iv = random_bytes(12);
    $sealed = openssl_encrypt($data, 'aes-256-gcm', $key, OPENSSL_RAW_DATA, $iv, $tag);
    $opened = openssl_decrypt($data, 'aes-256-gcm', $key, OPENSSL_RAW_DATA, $iv, $tag);
    $named = openssl_decrypt($data, 'chacha20-poly1305', $key, iv: $iv, tag: $tag);
    $cbc = openssl_encrypt($data, 'aes-256-cbc', $key, 0, $iv);
    $runtime = openssl_encrypt($data, $cipher, $key, 0, $iv);
    $noIv = openssl_encrypt($data, 'aes-256-cbc', $key, 0, '');
    return $sealed . $opened . $named . $cbc . $runtime . $noIv;
}
