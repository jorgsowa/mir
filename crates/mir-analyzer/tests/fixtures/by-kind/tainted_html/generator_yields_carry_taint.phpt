===description===
Values a generator yields (and the keys it yields them under) carry their taint
to whoever iterates it, through a user function or method, a declared
`Generator` return type, `yield from`, a stored generator or `iterator_to_array`.
A generator's `return` value, clean and sanitized yields stay unreported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fromGet() {
    yield $_GET['a'];
}

function keyed() {
    yield $_GET['k'] => 'value';
}

function clean() {
    yield 'literal';
}

function escaped() {
    yield htmlspecialchars($_GET['a']);
}

function wrap($x) {
    yield $x;
}

function returnOnly() {
    yield 1;
    return $_GET['a'];
}

function declared(): Generator {
    yield $_GET['a'];
}

function declaredGeneric(): \Generator {
    yield 'k' => $_GET['a'];
}

function delegatesToArray() {
    yield from [$_GET['a']];
}

function delegatesToGenerator() {
    yield from fromGet();
}

function delegatesToClean() {
    yield from clean();
}

function mixedYields($c) {
    yield 'safe';
    if ($c) {
        yield $_GET['a'];
    }
}

class Source {
    public function items() {
        yield $_GET['a'];
    }

    public static function all(): iterable {
        yield $_GET['a'];
    }

    public function viaProperty(string $p) {
        yield $p;
    }
}

function iterateValue() {
    foreach (fromGet() as $v) {
        echo $v;
    }
}

function iterateKey() {
    foreach (keyed() as $k => $v) {
        echo $k;
    }
}

function iterateKeyedValueIsClean() {
    foreach (keyed() as $k => $v) {
        echo $v;
    }
}

function iterateClean() {
    foreach (clean() as $v) {
        echo $v;
    }
}

function iterateEscaped() {
    foreach (escaped() as $v) {
        echo $v;
    }
}

function iterateWrapTainted() {
    foreach (wrap($_GET['a']) as $v) {
        echo $v;
    }
}

function iterateWrapClean() {
    foreach (wrap('literal') as $v) {
        echo $v;
    }
}

function iterateReturnOnly() {
    foreach (returnOnly() as $v) {
        echo $v;
    }
}

function iterateDeclared() {
    foreach (declared() as $v) {
        echo $v;
    }
}

function iterateDeclaredGeneric() {
    foreach (declaredGeneric() as $v) {
        echo $v;
    }
}

function iterateDelegatedArray() {
    foreach (delegatesToArray() as $v) {
        echo $v;
    }
}

function iterateDelegatedGenerator() {
    foreach (delegatesToGenerator() as $v) {
        echo $v;
    }
}

function iterateDelegatedClean() {
    foreach (delegatesToClean() as $v) {
        echo $v;
    }
}

function iterateMixed() {
    foreach (mixedYields(true) as $v) {
        echo $v;
    }
}

function iterateStored() {
    $g = fromGet();
    foreach ($g as $v) {
        echo $v;
    }
}

function currentValue() {
    $g = fromGet();
    echo $g->current();
}

function toArray() {
    foreach (iterator_to_array(fromGet()) as $v) {
        echo $v;
    }
}

function iterateMethod() {
    foreach ((new Source)->items() as $v) {
        echo $v;
    }
}

function iterateStaticMethod() {
    foreach (Source::all() as $v) {
        echo $v;
    }
}

function iterateMethodArgument() {
    foreach ((new Source)->viaProperty($_GET['a']) as $v) {
        echo $v;
    }
}

function iterateMethodCleanArgument() {
    foreach ((new Source)->viaProperty('literal') as $v) {
        echo $v;
    }
}
===expect===
TaintedHtml@70:8-70:16: Tainted HTML output — possible XSS
TaintedHtml@76:8-76:16: Tainted HTML output — possible XSS
TaintedHtml@100:8-100:16: Tainted HTML output — possible XSS
TaintedHtml@118:8-118:16: Tainted HTML output — possible XSS
TaintedHtml@124:8-124:16: Tainted HTML output — possible XSS
TaintedHtml@130:8-130:16: Tainted HTML output — possible XSS
TaintedHtml@136:8-136:16: Tainted HTML output — possible XSS
TaintedHtml@148:8-148:16: Tainted HTML output — possible XSS
TaintedHtml@155:8-155:16: Tainted HTML output — possible XSS
TaintedHtml@161:4-161:23: Tainted HTML output — possible XSS
TaintedHtml@166:8-166:16: Tainted HTML output — possible XSS
TaintedHtml@172:8-172:16: Tainted HTML output — possible XSS
TaintedHtml@178:8-178:16: Tainted HTML output — possible XSS
TaintedHtml@184:8-184:16: Tainted HTML output — possible XSS
