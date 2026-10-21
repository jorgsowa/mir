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
    }
}

function unguardedKeyIsTainted(): void {
    $s = [];
    foreach ($_GET as $k => $v) {
        $s[$k] = 1;
    }
    foreach ($s as $key => $_) {
        echo $key;
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
    }
    foreach (returnsKeysOfItsArgument($_GET) as $name => $_) {
        echo $name;
    }
    foreach (returnsOneOfTwo(true) as $name => $_) {
        echo $name;
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
    }
    $b = $merged;
    $b += $_GET;
    foreach ($b as $k => $_) {
        echo $k;
    }
    $c = $merged;
    $c = $_GET;
    foreach ($c as $k => $_) {
        echo $k;
    }
    $d = [...$_GET];
    foreach ($d as $k => $_) {
        echo $k;
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
}

function closureByReferenceCanAddKeys(): void {
    $s = [];
    $s['fixed'] = $_GET['v'];
    $add = function () use (&$s) {
        $s[$_GET['k']] = 1;
    };
    $add();
    echo implode(',', array_keys($s));
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
    }
    $u = ['a' => $_GET];
    foreach ($u as $key => $nested) {
        echo $key;
        foreach ($nested as $inner => $_) {
            echo $inner;
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
    }
}

function staticStateIsUnknown(): void {
    static $seen = [];
    foreach ($seen as $key => $_) {
        echo $key;
    }
    $seen[$_GET['k']] = 1;
}
===expect===
TaintedHtml@33:8-33:20: Tainted HTML output — possible XSS
TaintedHtml@43:8-43:18: Tainted HTML output — possible XSS
TaintedHtml@55:8-55:18: Tainted HTML output — possible XSS
TaintedHtml@70:4-70:25: Tainted HTML output — possible XSS
TaintedHtml@84:8-84:20: Tainted HTML output — possible XSS
TaintedHtml@128:12-128:23: Tainted HTML output — possible XSS
TaintedHtml@141:8-141:19: Tainted HTML output — possible XSS
TaintedHtml@178:8-178:19: Tainted HTML output — possible XSS
TaintedHtml@203:8-203:19: Tainted HTML output — possible XSS
TaintedHtml@206:8-206:19: Tainted HTML output — possible XSS
TaintedHtml@209:8-209:19: Tainted HTML output — possible XSS
TaintedHtml@222:8-222:16: Tainted HTML output — possible XSS
TaintedHtml@227:8-227:16: Tainted HTML output — possible XSS
TaintedHtml@232:8-232:16: Tainted HTML output — possible XSS
TaintedHtml@236:8-236:16: Tainted HTML output — possible XSS
TaintedHtml@250:4-250:38: Tainted HTML output — possible XSS
TaintedHtml@260:4-260:38: Tainted HTML output — possible XSS
TaintedHtml@273:8-273:18: Tainted HTML output — possible XSS
TaintedHtml@285:8-285:18: Tainted HTML output — possible XSS
TaintedHtml@303:8-303:18: Tainted HTML output — possible XSS
TaintedHtml@309:12-309:24: Tainted HTML output — possible XSS
TaintedHtml@323:12-323:24: Tainted HTML output — possible XSS
TaintedHtml@336:8-336:18: Tainted HTML output — possible XSS
