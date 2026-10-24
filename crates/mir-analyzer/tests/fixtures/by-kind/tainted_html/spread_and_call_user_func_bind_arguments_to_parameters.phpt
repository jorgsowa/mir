===description===
A spread of an array literal, and call_user_func()/call_user_func_array(),
bind each argument to its own parameter, by position or by name, so only the
parameters that reach a sink or the return value carry the taint.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
    <MissingClosureParamType errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedClosureParam errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function showBody(string $title, string $body): void { echo $body; }
function showTitle(string $title, string $body): void { echo $title; }
function pickBody(string $title, string $body): string { return $body; }
function pickTitle(string $title, string $body): string { return $title; }
function joinAll(string ...$parts): string { return implode(',', $parts); }
function showDefaulted(string $a, string $b = 'b', string $c = 'c'): void { echo $c; }

function spreadNamed(): void {
    $t = $_GET['t'];
    showBody(...['title' => $t, 'body' => 'b']);
    showBody(...['body' => 'b', 'title' => $t]);
    showTitle(...['title' => $t, 'body' => 'b']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    showBody(...['title' => 't', 'body' => $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function spreadPositional(): void {
    $t = $_GET['t'];
    showBody(...[$t, 'b']);
    showBody(...['b', $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    showTitle(...[$t, 'b']);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    showTitle(...['b', $t]);
}

function spreadAfterPositional(): void {
    $t = $_GET['t'];
    showBody($t, ...['b']);
    showBody('a', ...[$t]);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    showDefaulted('a', ...['c' => $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    showDefaulted($t, ...['c' => 'c']);
}

function spreadInResult(): void {
    $t = $_GET['t'];
    echo pickBody(...['title' => $t, 'body' => 'b']);
    echo pickBody(...[$t, 'b']);
    echo pickTitle(...[$t, 'b']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo pickTitle(...['body' => $t, 'title' => 't']);
    echo pickBody(...['body' => $t, 'title' => 't']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function spreadUnknownStaysConservative(array $args): void {
    $t = $_GET['t'];
    showBody(...$args);
    showBody(...[$t, ...$args]);
    $spread = ['title' => $t, 'body' => 'b'];
    showBody(...$spread);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    showBody(...[$args['k'] => $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    showBody(...[...$args, $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    showTitle(...[...$args, $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function spreadIntoVariadic(): void {
    $t = $_GET['t'];
    echo joinAll(...['a', 'b']);
    echo joinAll('a', ...[$t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo joinAll(...['x' => $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function closuresBindSpread(): void {
    $t = $_GET['t'];
    $body = function (string $title, string $body) { echo $body; };
    $body(...['title' => $t, 'body' => 'b']);
    $body(...[$t, 'b']);
    $body(...['b', $t]);
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $pick = fn (string $a, string $b) => $b;
    echo $pick(...['a' => $t, 'b' => 'b']);
    echo $pick(...['b' => $t, 'a' => 'a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callUserFuncPositional(): void {
    $t = $_GET['t'];
    call_user_func('showBody', $t, 'x');
    call_user_func('showTitle', $t, 'x');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    call_user_func('showBody', 'x', $t);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func('pickBody', $t, 'x');
    echo call_user_func('pickTitle', $t, 'x');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func('pickBody', 'x', $t);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callUserFuncArrayList(): void {
    $t = $_GET['t'];
    call_user_func_array('showBody', [$t, 'x']);
    call_user_func_array('showBody', ['x', $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func_array('pickBody', [$t, 'x']);
    echo call_user_func_array('pickTitle', [$t, 'x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callUserFuncArrayNamed(): void {
    $t = $_GET['t'];
    call_user_func_array('showBody', ['title' => $t, 'body' => 'x']);
    call_user_func_array('showBody', ['body' => $t, 'title' => 'x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func_array('pickBody', ['body' => 'x', 'title' => $t]);
    echo call_user_func_array('pickTitle', ['body' => 'x', 'title' => $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callUserFuncClosures(): void {
    $t = $_GET['t'];
    $show = function (string $a, string $b) { echo $b; };
    call_user_func($show, $t, 'x');
    call_user_func($show, 'x', $t);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    call_user_func_array($show, ['a' => $t, 'b' => 'x']);
    call_user_func_array($show, ['b' => $t, 'a' => 'x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func(fn ($a, $b) => $a, $t, 'x');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func(fn ($a, $b) => $b, $t, 'x');
}

function callUserFuncSanitizers(): void {
    $t = $_GET['t'];
    echo call_user_func('htmlspecialchars', $t);
    echo call_user_func('trim', $t);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callUserFuncVariadic(): void {
    $t = $_GET['t'];
    echo call_user_func('joinAll', 'a', $t);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func_array('joinAll', ['a', $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func('joinAll', 'a', 'b');
}

function callUserFuncUnknownStaysConservative(array $args, string $name): void {
    $t = $_GET['t'];
    echo call_user_func_array('pickBody', $args);
    echo call_user_func_array('pickBody', [$t, ...$args]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func_array('pickBody', ['nope' => $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func($name, $t);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo call_user_func_array([$name, 'run'], [$t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callUserFuncDefaults(): void {
    $t = $_GET['t'];
    call_user_func('showDefaulted', 'a', 'b', $t);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    call_user_func('showDefaulted', $t);
    call_user_func_array('showDefaulted', ['a', 'c' => $t]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
