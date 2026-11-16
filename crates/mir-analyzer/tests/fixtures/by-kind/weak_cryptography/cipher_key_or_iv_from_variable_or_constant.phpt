===description===
A cipher, key or IV held in a variable or class constant with one literal value
is checked like one written at the call; values only known at runtime aren't.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <PossiblyInvalidArgument errorLevel="suppress"/>
    <PossiblyFalseArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Vault {
    private const CIPHER = 'aes-128-ecb';

    public function seal(string $data, string $key, string $cipher): void {
        openssl_encrypt($data, self::CIPHER, $key);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WeakCryptography: Weak cryptography: ECB mode of 'aes-128-ecb' leaks patterns of the plaintext
        $legacy = 'des-cbc';
        openssl_encrypt($data, $legacy, $key, 0, random_bytes(8));
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WeakCryptography: Weak cryptography: cipher 'des-cbc' is broken
        $secret = 'static-key';
        openssl_encrypt($data, 'aes-256-cbc', $secret, 0, random_bytes(16));
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WeakCryptography: Weak cryptography: the key is hardcoded
        openssl_encrypt($data, $cipher, $key, 0, random_bytes(16));
    }
}
