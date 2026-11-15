===description===
Comparing the result with `1`, or `> 0`, rejects the `-1` error value.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function accept(string $data, string $signature, OpenSSLAsymmetricKey $key, OpenSSLCertificate $cert): bool {
    if (openssl_verify($data, $signature, $key) === 1) {
        return true;
    }
    if (openssl_verify($data, $signature, $key) == 1 || openssl_x509_verify($cert, $key) > 0) {
        return true;
    }
    return false;
}
