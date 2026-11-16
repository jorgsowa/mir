===description===
A token built from the clock is as guessable as one from `mt_rand()`, and
names with acronyms (`getOTP`) are split into words like any other.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function makeResetToken(): string {
    return md5((string) microtime(true));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureRandomness: Insecure randomness: the value makeResetToken() returns comes from a non-cryptographic generator; use random_bytes() or random_int()
}

function getOTP(): string {
    return (string) mt_rand(100000, 999999);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureRandomness: Insecure randomness: the value getOTP() returns comes from a non-cryptographic generator; use random_bytes() or random_int()
}

function sessionToken(): string {
    return sha1((string) time());
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureRandomness: Insecure randomness: the value sessionToken() returns comes from a non-cryptographic generator; use random_bytes() or random_int()
}
