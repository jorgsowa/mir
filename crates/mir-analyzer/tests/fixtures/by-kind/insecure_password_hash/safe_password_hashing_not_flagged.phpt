===description===
Default or strong `password_hash()` options, encoded or hex pre-hashes, Argon2
with a raw pre-hash, and digests computed outside `password_hash()` (pre-hashes
built in a separate step, API signatures) are fine.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function safe(string $password, string $passwordHash, int $cost): void {
    password_hash($password, PASSWORD_DEFAULT);
    password_hash($password, PASSWORD_BCRYPT, ['cost' => 12]);
    password_hash($password, PASSWORD_BCRYPT, ['cost' => $cost]);
    password_hash(base64_encode(hash('sha256', $password, true)), PASSWORD_BCRYPT);
    password_hash(hash('sha256', $password), PASSWORD_BCRYPT);
    password_hash(hash('sha256', $password, true), PASSWORD_ARGON2ID);
    password_hash($password, PASSWORD_ARGON2ID, ['memory_cost' => 65536, 'time_cost' => 4]);
    password_verify(hash('sha384', $password), $passwordHash);
    $preHashed = base64_encode(hash('sha512', $password, true));
    password_hash($preHashed, PASSWORD_BCRYPT);
    $signature = md5('payload' . $password);
}
