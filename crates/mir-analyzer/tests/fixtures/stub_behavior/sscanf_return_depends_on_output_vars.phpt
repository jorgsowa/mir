===description===
`sscanf` returns an array (or null) without output variables and an int (or null) with them.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function arrayForm(string $s): int {
    $r = sscanf($s, '%d');
    /** @mir-check $r is array|null */
    if ($r === null) {
        return 0;
    }
    return (int) $r[0];
}
function varsForm(string $s): int {
    $n = sscanf($s, '%d %s', $a, $b);
    /** @mir-check $n is int|null */
    return ($n ?? 0) + 1;
}
function spreadForm(string $s, array $vars): void {
    $x = sscanf($s, '%d', ...$vars);
    /** @mir-check $x is array|int|null */
}
