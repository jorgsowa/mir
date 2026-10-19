===description===
A return value or property write whose type is a literal or numeric (after `in_array`,
`ctype_digit`, `is_numeric` or a cast) carries no taint into the callee's summary.
Unguarded values and mixed branches still do.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function whitelisted(string $s): string {
    return in_array($s, ['name', 'date'], true) ? $s : 'name';
}

function earlyReturn(string $s): string {
    if (!in_array($s, ['name', 'date'], true)) {
        return 'name';
    }
    return $s;
}

function castToInt(string $s): int {
    return (int) $s;
}

function numeric(string $s): string {
    return is_numeric($s) ? $s : '0';
}

function unguarded(string $s): string {
    return $s;
}

function partlyGuarded(string $s, bool $flag): string {
    return $flag ? $s : 'name';
}

function elseBranchLeaks(string $s): string {
    if (in_array($s, ['name', 'date'], true)) {
        return $s;
    }
    return $s . 'x';
}

function queries(PDO $db): void {
    $db->query('ORDER BY ' . whitelisted($_GET['s']));
    $db->query('ORDER BY ' . earlyReturn($_GET['s']));
    $db->query('LIMIT ' . castToInt($_GET['n']));
    $db->query('LIMIT ' . numeric($_GET['n']));
    $db->query('ORDER BY ' . unguarded($_GET['s']));
    $db->query('ORDER BY ' . partlyGuarded($_GET['s'], true));
    $db->query('ORDER BY ' . elseBranchLeaks($_GET['s']));
}

class Page {
    private string $page = '1';
    private string $raw = '';
    private string $mixed = '';
    private int $count = 0;

    public function __construct(string $p, string $q) {
        $this->page = ctype_digit($p) ? $p : '1';
        $this->raw = $q;
        $this->mixed = 'a';
        if ($q !== '') {
            $this->mixed = $q;
        }
        $this->count = (int) $p;
    }

    public function showPage(): void {
        echo $this->page;
    }

    public function showRaw(): void {
        echo $this->raw;
    }

    public function showMixed(): void {
        echo $this->mixed;
    }

    public function showCount(): void {
        echo $this->count;
    }
}

function pages(): void {
    $p = new Page($_GET['p'], $_GET['q']);
    $p->showPage();
    $p->showRaw();
    $p->showMixed();
    $p->showCount();
}

function ternaryGuards(PDO $db): void {
    $a = $_GET['a'];
    $b = $_GET['b'];
    $db->query('id = ' . (ctype_digit($a) ? $a : '1'));
    $db->query('id = ' . (!ctype_digit($a) ? '1' : $a));
    $db->query('id = ' . (ctype_digit($a) && ctype_digit($b) ? $a . $b : '1'));
    $db->query('id = ' . (ctype_digit($a) ? $b : '1'));
    $db->query('id = ' . (ctype_digit($a) ? '1' : $a));
    $db->query('id = ' . (ctype_digit($a) || $b !== '' ? $a : '1'));
}

function guardDoesNotOutliveTernary(PDO $db): void {
    $a = $_GET['a'];
    $db->query('id = ' . (ctype_digit($a) ? $a : '1'));
    $db->query('id = ' . $a);
}

function digitsOrDefault(string $p): string {
    return ctype_digit($p) ? $p : '1';
}

function digitsFallThrough(string $p): string {
    return !ctype_digit($p) ? '1' : $p;
}

function ternaryReturns(PDO $db): void {
    $db->query('id = ' . digitsOrDefault($_GET['p']));
    $db->query('id = ' . digitsFallThrough($_GET['p']));
}
===expect===
TaintedSql@41:4-41:51: Tainted SQL query — possible SQL injection
TaintedSql@42:4-42:61: Tainted SQL query — possible SQL injection
TaintedSql@43:4-43:57: Tainted SQL query — possible SQL injection
TaintedHtml@82:4-82:17: Tainted HTML output — possible XSS
TaintedHtml@83:4-83:19: Tainted HTML output — possible XSS
TaintedSql@93:4-93:54: Tainted SQL query — possible SQL injection
TaintedSql@94:4-94:54: Tainted SQL query — possible SQL injection
TaintedSql@95:4-95:67: Tainted SQL query — possible SQL injection
TaintedSql@101:4-101:28: Tainted SQL query — possible SQL injection
