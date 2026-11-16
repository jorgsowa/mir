===description===
A verification result stored in a variable first still lets its -1 error through
a truthiness check; `openssl_pkcs7_verify()` and `ldap_compare()` succeed with
`true`, not 1.
===file===
<?php
function stored(string $data, string $signature, OpenSSLAsymmetricKey $key): bool {
    $ok = openssl_verify($data, $signature, $key, OPENSSL_ALGO_SHA256);
    if ($ok) {
//      ^^^ InsecureCheckResult: Insecure check: $ok, a verification result, is -1 on error, which passes this check; compare with === 1
        return true;
    }
    return false;
}

function storedStrict(string $data, string $signature, OpenSSLAsymmetricKey $key): bool {
    $ok = openssl_verify($data, $signature, $key, OPENSSL_ALGO_SHA256);
    return $ok === 1;
}

function signedMail(string $file): bool {
    if (openssl_pkcs7_verify($file, 0)) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCheckResult: Insecure check: openssl_pkcs7_verify() is -1 on error, which passes this check; compare with === true
        return true;
    }
    return false;
}

function sameAttribute(\LDAP\Connection $ldap, string $dn): bool {
    $result = ldap_compare($ldap, $dn, 'memberOf', 'admins');
    if ($result != 0) {
//      ^^^^^^^^^^^^ InsecureCheckResult: Insecure check: $result, a verification result, is -1 on error, which passes this check; compare with === true
        return true;
    }
    return false;
}
