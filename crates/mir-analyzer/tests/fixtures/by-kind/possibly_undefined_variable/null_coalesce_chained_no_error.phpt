===description===
no PossiblyUndefinedVariable for $decrypted in chained if-with-?? fallback
===file===
<?php
function decrypt(bool $aead): string {
    if ($aead) {
        $decrypted = openssl_decrypt('foo', 'AES-256-GCM', 'key', 0, 'iv', 'tag');
    }
    if (($decrypted ?? false) === false) {
        $decrypted = openssl_decrypt('foo', 'AES-256-CBC', 'key', 0, 'iv');
    }
    if (($decrypted ?? false) === false) {
        throw new \RuntimeException('Decryption failed');
    }
    return $decrypted;
}
===expect===
