===description===
`hash_equals()`, a digest compared with a literal, request data compared with
values not named for a secret, two stored values compared with each other and
a parameter named for a secret (usually the submitted value, as in a
double-submit check) aren't reported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <PossiblyUndefinedArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function verify(string $payload, string $key, string $storedToken, string $user): bool {
    $expected = hash_hmac('sha256', $payload, $key);
    if (!hash_equals($expected, $_SERVER['HTTP_X_SIGNATURE'] ?? '')) {
        return false;
    }
    if ($expected === '') {
        return false;
    }
    if ($_GET['user'] === $user) {
        return true;
    }
    if ($_POST['token'] === '') {
        return false;
    }
    return $storedToken === $user;
}

function doubleSubmit(string $token): bool {
    return ($_SERVER['HTTP_X_CSRF_TOKEN'] ?? '') === $token;
}
