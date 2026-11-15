===description===
`preg_match()` returns false when matching fails, so input with invalid UTF-8 for a
`/u` pattern, or long enough to hit the backtracking limit in a quantified group,
skips a rejection guarded by a truthy match.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function comment(): void {
    $text = $_POST['text'];
    if (preg_match('/<script/iu', $text)) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCheckResult: Insecure check: preg_match() returns false on invalid UTF-8, which skips this rejection; compare with !== 0
        exit('rejected');
    }
    echo htmlspecialchars($text);
}

function query(): void {
    $q = $_GET['q'];
    if (preg_match('/(union|select)+\s/i', $q) === 1 || str_contains($q, '--')) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureCheckResult: Insecure check: preg_match() returns false when a long input hits the backtracking limit, which skips this rejection; compare with !== 0
        throw new InvalidArgumentException('bad query');
    }
}
