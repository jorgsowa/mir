===description===
An allowlist match that rejects on a failed match, `!== 0`, a pattern that can't
fail, a branch that doesn't reject, and program data aren't reported.
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
function comment(string $config): void {
    $text = (string) $_POST['text'];
    if (!preg_match('/^[\p{L} ]+$/u', $text)) {
        exit('rejected');
    }
    if (preg_match('/<script/iu', $text) !== 0) {
        exit('rejected');
    }
    if (preg_match('/<script/i', $text)) {
        exit('rejected');
    }
    if (preg_match('/(union|select)+/u', $text)) {
        echo 'looks like SQL';
    }
    if (preg_match('/(a|b)+/u', $config)) {
        return;
    }
    echo htmlspecialchars($text);
}
