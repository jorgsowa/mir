===description===
The key names of an array built element by element (`$s = []; $s[$k] = $v`)
are tracked apart from its values: a request value stored under a validated key
leaves the keys clean for `foreach`, `array_keys()` and callers of the function
that returns it. Anything that can add other keys (a reference, a merge, an
unknown callee or implementation, a tainted key) keeps them tainted.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
    <PossiblyUndefinedArrayOffset errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function collect(): array {
    $found = [];
    foreach ($_SERVER as $k => $v) {
        if (preg_match('/^\w+$/D', $k)) {
            $found[$k] = is_string($v) ? $v : null;
        }
    }
    return $found;
}

function inTheSameFunction(): void {
    $s = [];
    foreach ($_GET as $k => $v) {
        if (preg_match('/^\w+$/D', $k) !== 1) {
            continue;
        }
        $s[$k] = $v;
    }
    foreach ($s as $key => $_) {
        echo $key;
    }
}

function valuesAreStillTainted(): void {
    $s = [];
    foreach ($_GET as $k => $v) {
        if (ctype_alnum($k)) {
            $s[$k] = $v;
        }
    }
    foreach ($s as $key => $value) {
        echo $value;
//      ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function unguardedKeyIsTainted(): void {
    $s = [];
    foreach ($_GET as $k => $v) {
        $s[$k] = 1;
    }
    foreach ($s as $key => $_) {
        echo $key;
//      ^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function guardThatProvesNothingKeepsTheKey(): void {
    $s = [];
    foreach ($_GET as $k => $v) {
        if (preg_match('/^.+$/D', $k)) {
            $s[$k] = $v;
        }
    }
    foreach ($s as $key => $_) {
        echo $key;
//      ^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function keyReaders(): void {
    $s = [];
    foreach ($_GET as $k => $v) {
        if (ctype_digit($k)) {
            $s[$k] = $v;
        }
    }
    echo implode(',', array_keys($s));
    echo array_key_first($s);
    echo array_key_last($s);
    echo key($s);
    echo json_encode($s);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo json_encode(array_keys($s));
}

function callerIteratesKeysOfReturnedArray(): void {
    foreach (collect() as $name => $value) {
        echo $name;
    }
    $all = collect();
    foreach ($all as $name => $value) {
        echo $name;
    }
    echo implode(',', array_keys(collect()));
    foreach (collect() as $name => $value) {
        echo $value;
//      ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

class Vault {
    public function list(): array {
        $s = [];
        foreach ($_SERVER as $k => $v) {
            if (ctype_alnum($k)) {
                $s[$k] = $v;
            }
        }
        return $s;
    }

    public static function all(): array {
        $s = [];
        foreach ($_SERVER as $k => $v) {
            if (ctype_alnum($k)) {
                $s[$k] = $v;
            }
        }
        return $s;
    }

    public function keysFromRequest(): array {
        $s = [];
        foreach ($_GET as $k => $v) {
            $s[$k] = $v;
        }
        return $s;
    }

    public function run(): void {
        foreach ($this->list() as $name => $_) {
            echo $name;
        }
        foreach (self::all() as $name => $_) {
            echo $name;
        }
        foreach (static::all() as $name => $_) {
            echo $name;
        }
        foreach ($this->keysFromRequest() as $name => $_) {
            echo $name;
//          ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        }
    }
}

function throughObjects(Vault $vault): void {
    foreach ($vault->list() as $name => $_) {
        echo $name;
    }
    foreach (Vault::all() as $name => $_) {
        echo $name;
    }
    foreach ($vault->keysFromRequest() as $name => $_) {
        echo $name;
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

abstract class Store {
    abstract public function entries(): array;
}

class CleanStore extends Store {
    public function entries(): array {
        $s = [];
        foreach ($_SERVER as $k => $v) {
            if (ctype_alnum($k)) {
                $s[$k] = $v;
            }
        }
        return $s;
    }
}

class DiskStore extends Store {
    public function entries(): array {
        return [];
    }
}

class RequestStore extends Store {
    public function entries(): array {
        return $_GET;
    }
}

function everyImplementationDecides(CleanStore $clean, Store $any): void {
    foreach ($clean->entries() as $name => $_) {
        echo $name;
    }
    foreach ($any->entries() as $name => $_) {
        echo $name;
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function returnsWhatItIsGiven(array $in): array {
    return $in;
}

function returnsKeysOfItsArgument(array $in): array {
    $s = [];
    foreach ($in as $k => $v) {
        $s[$k] = 1;
    }
    return $s;
}

function returnsOneOfTwo(bool $c): array {
    if ($c) {
        return collect();
    }
    return $_GET;
}

function callsWithUnknownKeys(): void {
    foreach (returnsWhatItIsGiven($_GET) as $name => $_) {
        echo $name;
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    foreach (returnsKeysOfItsArgument($_GET) as $name => $_) {
        echo $name;
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    foreach (returnsOneOfTwo(true) as $name => $_) {
        echo $name;
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function otherWaysToAddKeys(array $extra): void {
    $merged = [];
    foreach ($_GET as $k => $v) {
        if (ctype_digit($k)) {
            $merged[$k] = $v;
        }
    }
    $a = array_merge($merged, $_GET);
    foreach ($a as $k => $_) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    $b = $merged;
    $b += $_GET;
    foreach ($b as $k => $_) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    $c = $merged;
    $c = $_GET;
    foreach ($c as $k => $_) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    $d = [...$_GET];
    foreach ($d as $k => $_) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    $e = [];
    $e[] = $_GET['x'];
    foreach ($e as $k => $_) {
        echo $k;
    }
}

function referencesCanAddKeys(): void {
    $s = [];
    $alias = &$s;
    $s['fixed'] = $_GET['v'];
    $alias[$_GET['k']] = 1;
    echo implode(',', array_keys($s));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function closureByReferenceCanAddKeys(): void {
    $s = [];
    $s['fixed'] = $_GET['v'];
    $add = function () use (&$s) {
        $s[$_GET['k']] = 1;
    };
    $add();
    echo implode(',', array_keys($s));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function noReferenceNoExtraKeys(): void {
    $s = [];
    $s['fixed'] = $_GET['v'];
    echo implode(',', array_keys($s));
}

function byReferenceArgumentCanAddKeys(): void {
    $s = [];
    parse_str($_SERVER['QUERY_STRING'], $s);
    foreach ($s as $key => $_) {
        echo $key;
//      ^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function branchesMerge(bool $c): void {
    $s = [];
    if ($c) {
        $s[$_GET['k']] = 1;
    } else {
        $s['fixed'] = 1;
    }
    foreach ($s as $key => $_) {
        echo $key;
//      ^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    $t = [];
    if ($c) {
        $t['fixed'] = $_GET['v'];
    }
    foreach ($t as $key => $_) {
        echo $key;
    }
}

function literalArrays(): void {
    $s = ['a' => $_GET['a'], 'b' => $_GET['b']];
    foreach ($s as $key => $_) {
        echo $key;
    }
    $t = [$_GET['k'] => 1];
    foreach ($t as $key => $_) {
        echo $key;
//      ^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    $u = ['a' => $_GET];
    foreach ($u as $key => $nested) {
        echo $key;
        foreach ($nested as $inner => $_) {
            echo $inner;
//          ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        }
    }
}

function keysOfValueArraysStayTainted(): void {
    $s = [];
    foreach ($_GET as $k => $v) {
        if (ctype_alnum($k)) {
            $s[$k] = $v;
        }
    }
    foreach ($s as $k => $nested) {
        foreach ($nested as $inner => $_) {
            echo $inner;
//          ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        }
    }
}

function loopsMerge(): void {
    $s = [];
    for ($i = 0; $i < 3; $i++) {
        if ($i === 2) {
            $s[$_GET['k']] = 1;
        }
    }
    foreach ($s as $key => $_) {
        echo $key;
//      ^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function staticStateIsUnknown(): void {
    static $seen = [];
    foreach ($seen as $key => $_) {
        echo $key;
    }
    $seen[$_GET['k']] = 1;
}
