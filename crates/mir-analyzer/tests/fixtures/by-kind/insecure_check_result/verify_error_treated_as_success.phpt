===description===
Signature checks that return `-1` on error, used as a boolean or compared loosely,
accept a signature that failed to verify.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function accept(string $data, string $signature, OpenSSLAsymmetricKey $key): bool {
    if (openssl_verify($data, $signature, $key)) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCheckResult: Insecure check: openssl_verify() returns -1 on error, which passes this check; compare with === 1
        return true;
    }
    return false;
}

function rejectInvalid(string $data, string $signature, OpenSSLAsymmetricKey $key): void {
    if (!openssl_verify($data, $signature, $key) && strlen($data) > 0) {
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCheckResult: Insecure check: openssl_verify() returns -1 on error, which passes this check; compare with === 1
        throw new RuntimeException('bad signature');
    }
}

function trusted(OpenSSLCertificate $cert, OpenSSLAsymmetricKey $key): bool {
    return openssl_x509_verify($cert, $key) != 0 ? true : false;
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCheckResult: Insecure check: openssl_x509_verify() returns -1 on error, which passes this check; compare with === 1
}
