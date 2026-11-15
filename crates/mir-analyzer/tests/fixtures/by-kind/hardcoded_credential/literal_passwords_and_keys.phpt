===description===
Literal passwords at known password parameters and literals shaped like live
secrets are credentials written in the source.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function connect(LDAP\Connection $ldap): void {
    $db = new PDO('mysql:host=db;dbname=app', 'app', 's3cr3t-pa55');
//        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HardcodedCredential: Hard-coded credential: the password passed to new PDO() is written in the source
    $link = mysqli_connect('localhost', 'root', 'hunter2');
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HardcodedCredential: Hard-coded credential: the password passed to mysqli_connect() is written in the source
    ldap_bind($ldap, 'cn=admin,dc=example,dc=org', 'Adm1nPass');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HardcodedCredential: Hard-coded credential: the password passed to ldap_bind() is written in the source
    $pg = pg_connect('host=db dbname=app user=app password=pgpass');
//        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HardcodedCredential: Hard-coded credential: the password in the pg_connect() connection string is written in the source
}

function cloud(): array {
    return [
        'key' => 'AKIAJ7QWERTYUIOPASDF',
//               ^^^^^^^^^^^^^^^^^^^^^^ HardcodedCredential: Hard-coded credential: an AWS access key ID is written in the source
        'stripe' => 'sk_live_51HxQwErTyUiOpAsDfGhJkLzX',
//                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HardcodedCredential: Hard-coded credential: a Stripe live secret key is written in the source
    ];
}

function signingKey(): string {
    return "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC7VJTUt9Us8cKjMzEfYyjiWA4R4\n-----END PRIVATE KEY-----\n";
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ HardcodedCredential: Hard-coded credential: a private key is written in the source
}
