===description===
A session ID from a guessable generator can be predicted by an attacker.
===file===
<?php
function start(): void {
    session_id(sha1(uniqid('', true)));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureRandomness: Insecure randomness: the session ID comes from a non-cryptographic generator; use random_bytes() or random_int()
    $id = 'sess' . mt_rand();
    session_id($id);
//  ^^^^^^^^^^^^^^^ InsecureRandomness: Insecure randomness: the session ID comes from a non-cryptographic generator; use random_bytes() or random_int()
}
