===description===
`password_hash()` with a `salt` option (ignored since PHP 8.0), a bcrypt cost
below 10, or a raw binary pre-hash that bcrypt truncates at a NUL byte.
===file===
<?php
function options(string $password): void {
    password_hash($password, PASSWORD_DEFAULT, ['salt' => 'pepper']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecurePasswordHash: Insecure password hashing: password_hash() ignores the 'salt' option since PHP 8.0
    password_hash($password, PASSWORD_BCRYPT, ['cost' => 4]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecurePasswordHash: Insecure password hashing: bcrypt cost 4 is below 10
    password_hash(hash('sha256', $password, true), PASSWORD_BCRYPT);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecurePasswordHash: Insecure password hashing: a raw binary digest can contain NUL bytes, where bcrypt stops reading
    password_hash(password: md5($password, true), algo: '2y');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecurePasswordHash: Insecure password hashing: a raw binary digest can contain NUL bytes, where bcrypt stops reading
}
