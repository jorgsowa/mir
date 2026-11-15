===description===
Credentials read at runtime, empty passwords, AWS's documented example key, key
headers without key material and secret-like but shorter strings aren't reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <PossiblyFalseArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function connect(string $password): void {
    $db = new PDO('mysql:host=db;dbname=app', 'app', (string) getenv('DB_PASSWORD'));
    $local = new PDO('mysql:host=127.0.0.1;dbname=test', 'root', '');
    $link = mysqli_connect('localhost', 'root', $password);
    $pg = pg_connect('host=db dbname=app user=app');
    $example = 'AKIAIOSFODNN7EXAMPLE';
    $isKey = str_starts_with($password, '-----BEGIN RSA PRIVATE KEY-----');
    $test = 'sk_test_51HxQwErTyUiOpAsDfGhJkLzX';
    $label = 'sk_live_mode_enabled';
}
