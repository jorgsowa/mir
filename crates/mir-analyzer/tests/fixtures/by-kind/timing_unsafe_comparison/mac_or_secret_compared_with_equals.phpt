===description===
A `hash_hmac()` digest, or request data compared with a stored token or
secret, checked with `==`/`===` leaks through its timing how much of the
value matched.
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
function webhook(string $payload, string $key): bool {
    $expected = hash_hmac('sha256', $payload, $key);
    if ($expected !== $_SERVER['HTTP_X_SIGNATURE']) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TimingUnsafeComparison: a hash_hmac() digest is compared with !==, which returns sooner the fewer leading characters match; use hash_equals()
        return false;
    }
    return base64_encode(hash_hmac('sha1', $payload, $key, true)) == $_GET['sig'];
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TimingUnsafeComparison: a hash_hmac() digest is compared with ==, which returns sooner the fewer leading characters match; use hash_equals()
}

function csrf(): void {
    if ($_POST['token'] !== $_SESSION['csrf_token']) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TimingUnsafeComparison: the stored secret 'csrf_token' is compared with !==, which returns sooner the fewer leading characters match; use hash_equals()
        exit;
    }
}

final class Api {
    private string $apiKey = '';

    public function allowed(): bool {
        $given = $_GET['key'] ?? '';
        return $given === $this->apiKey;
//             ^^^^^^^^^^^^^^^^^^^^^^^^ TimingUnsafeComparison: the stored secret 'apiKey' is compared with ===, which returns sooner the fewer leading characters match; use hash_equals()
    }
}
