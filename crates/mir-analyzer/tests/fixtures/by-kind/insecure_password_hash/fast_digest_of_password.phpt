===description===
A password hashed with `md5()`, `sha1()`, `crc32()` or `hash()` is guessed
quickly once the hash leaks. A pre-hash inside the argument of `password_hash()` or `password_verify()`, a raw
binary digest, a password mixed into other text, and digests of values not named
for a password, aren't reported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function login(): string {
    $pass = $_GET['password'];
    $pass = md5($pass);
//          ^^^^^^^^^^ InsecurePasswordHash: Insecure password hashing: md5() is a fast hash, so a leaked password hash is guessed quickly; use password_hash()
    return $pass;
}

function change(string $pass_new): string {
    return sha1($pass_new);
//         ^^^^^^^^^^^^^^^ InsecurePasswordHash: Insecure password hashing: sha1() is a fast hash, so a leaked password hash is guessed quickly; use password_hash()
}

function register(): string {
    return hash('sha256', (string) $_POST['password']);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecurePasswordHash: Insecure password hashing: hash() is a fast hash, so a leaked password hash is guessed quickly; use password_hash()
}

function fromProperty(object $user): string {
    return md5($user->password);
//         ^^^^^^^^^^^^^^^^^^^^ InsecurePasswordHash: Insecure password hashing: md5() is a fast hash, so a leaked password hash is guessed quickly; use password_hash()
}

function preHashed(string $password, string $stored): bool {
    password_hash(base64_encode(hash('sha256', $password, true)), PASSWORD_BCRYPT);
    return password_verify(hash('sha256', $password), $stored);
}

function signature(string $password): string {
    return md5('payload' . $password);
}

function avatar(string $email): string {
    return md5(strtolower($email));
}

function quote(string $passage): string {
    return md5($passage);
}
