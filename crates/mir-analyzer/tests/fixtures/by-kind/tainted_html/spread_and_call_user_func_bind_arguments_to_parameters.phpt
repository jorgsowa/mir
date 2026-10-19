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
    showBody(...['title' => 't', 'body' => $t]);
}

function spreadPositional(): void {
    $t = $_GET['t'];
    showBody(...[$t, 'b']);
    showBody(...['b', $t]);
    showTitle(...[$t, 'b']);
    showTitle(...['b', $t]);
}

function spreadAfterPositional(): void {
    $t = $_GET['t'];
    showBody($t, ...['b']);
    showBody('a', ...[$t]);
    showDefaulted('a', ...['c' => $t]);
    showDefaulted($t, ...['c' => 'c']);
}

function spreadInResult(): void {
    $t = $_GET['t'];
    echo pickBody(...['title' => $t, 'body' => 'b']);
    echo pickBody(...[$t, 'b']);
    echo pickTitle(...[$t, 'b']);
    echo pickTitle(...['body' => $t, 'title' => 't']);
    echo pickBody(...['body' => $t, 'title' => 't']);
}

function spreadUnknownStaysConservative(array $args): void {
    $t = $_GET['t'];
    showBody(...$args);
    showBody(...[$t, ...$args]);
    $spread = ['title' => $t, 'body' => 'b'];
    showBody(...$spread);
    showBody(...[$args['k'] => $t]);
    showBody(...[...$args, $t]);
    showTitle(...[...$args, $t]);
}

function spreadIntoVariadic(): void {
    $t = $_GET['t'];
    echo joinAll(...['a', 'b']);
    echo joinAll('a', ...[$t]);
    echo joinAll(...['x' => $t]);
}

function closuresBindSpread(): void {
    $t = $_GET['t'];
    $body = function (string $title, string $body) { echo $body; };
    $body(...['title' => $t, 'body' => 'b']);
    $body(...[$t, 'b']);
    $body(...['b', $t]);
    $pick = fn (string $a, string $b) => $b;
    echo $pick(...['a' => $t, 'b' => 'b']);
    echo $pick(...['b' => $t, 'a' => 'a']);
}

function callUserFuncPositional(): void {
    $t = $_GET['t'];
    call_user_func('showBody', $t, 'x');
    call_user_func('showTitle', $t, 'x');
    call_user_func('showBody', 'x', $t);
    echo call_user_func('pickBody', $t, 'x');
    echo call_user_func('pickTitle', $t, 'x');
    echo call_user_func('pickBody', 'x', $t);
}

function callUserFuncArrayList(): void {
    $t = $_GET['t'];
    call_user_func_array('showBody', [$t, 'x']);
    call_user_func_array('showBody', ['x', $t]);
    echo call_user_func_array('pickBody', [$t, 'x']);
    echo call_user_func_array('pickTitle', [$t, 'x']);
}

function callUserFuncArrayNamed(): void {
    $t = $_GET['t'];
    call_user_func_array('showBody', ['title' => $t, 'body' => 'x']);
    call_user_func_array('showBody', ['body' => $t, 'title' => 'x']);
    echo call_user_func_array('pickBody', ['body' => 'x', 'title' => $t]);
    echo call_user_func_array('pickTitle', ['body' => 'x', 'title' => $t]);
}

function callUserFuncClosures(): void {
    $t = $_GET['t'];
    $show = function (string $a, string $b) { echo $b; };
    call_user_func($show, $t, 'x');
    call_user_func($show, 'x', $t);
    call_user_func_array($show, ['a' => $t, 'b' => 'x']);
    call_user_func_array($show, ['b' => $t, 'a' => 'x']);
    echo call_user_func(fn ($a, $b) => $a, $t, 'x');
    echo call_user_func(fn ($a, $b) => $b, $t, 'x');
}

function callUserFuncSanitizers(): void {
    $t = $_GET['t'];
    echo call_user_func('htmlspecialchars', $t);
    echo call_user_func('trim', $t);
}

function callUserFuncVariadic(): void {
    $t = $_GET['t'];
    echo call_user_func('joinAll', 'a', $t);
    echo call_user_func_array('joinAll', ['a', $t]);
    echo call_user_func('joinAll', 'a', 'b');
}

function callUserFuncUnknownStaysConservative(array $args, string $name): void {
    $t = $_GET['t'];
    echo call_user_func_array('pickBody', $args);
    echo call_user_func_array('pickBody', [$t, ...$args]);
    echo call_user_func_array('pickBody', ['nope' => $t]);
    echo call_user_func($name, $t);
    echo call_user_func_array([$name, 'run'], [$t]);
}

function callUserFuncDefaults(): void {
    $t = $_GET['t'];
    call_user_func('showDefaulted', 'a', 'b', $t);
    call_user_func('showDefaulted', $t);
    call_user_func_array('showDefaulted', ['a', 'c' => $t]);
}
===expect===
TaintedHtml@14:4-14:48: Tainted HTML output — possible XSS
TaintedHtml@15:4-15:47: Tainted HTML output — possible XSS
TaintedHtml@21:4-21:26: Tainted HTML output — possible XSS
TaintedHtml@22:4-22:27: Tainted HTML output — possible XSS
TaintedHtml@29:4-29:26: Tainted HTML output — possible XSS
TaintedHtml@30:4-30:38: Tainted HTML output — possible XSS
TaintedHtml@38:4-38:33: Tainted HTML output — possible XSS
TaintedHtml@40:4-40:53: Tainted HTML output — possible XSS
TaintedHtml@48:4-48:24: Tainted HTML output — possible XSS
TaintedHtml@49:4-49:35: Tainted HTML output — possible XSS
TaintedHtml@50:4-50:31: Tainted HTML output — possible XSS
TaintedHtml@51:4-51:32: Tainted HTML output — possible XSS
TaintedHtml@57:4-57:31: Tainted HTML output — possible XSS
TaintedHtml@58:4-58:33: Tainted HTML output — possible XSS
TaintedHtml@66:4-66:23: Tainted HTML output — possible XSS
TaintedHtml@69:4-69:43: Tainted HTML output — possible XSS
TaintedHtml@75:4-75:40: Tainted HTML output — possible XSS
TaintedHtml@76:4-76:39: Tainted HTML output — possible XSS
TaintedHtml@78:4-78:46: Tainted HTML output — possible XSS
TaintedHtml@79:4-79:45: Tainted HTML output — possible XSS
TaintedHtml@85:4-85:47: Tainted HTML output — possible XSS
TaintedHtml@87:4-87:54: Tainted HTML output — possible XSS
TaintedHtml@93:4-93:68: Tainted HTML output — possible XSS
TaintedHtml@95:4-95:75: Tainted HTML output — possible XSS
TaintedHtml@102:4-102:34: Tainted HTML output — possible XSS
TaintedHtml@104:4-104:56: Tainted HTML output — possible XSS
TaintedHtml@105:4-105:52: Tainted HTML output — possible XSS
TaintedHtml@112:4-112:36: Tainted HTML output — possible XSS
TaintedHtml@117:4-117:44: Tainted HTML output — possible XSS
TaintedHtml@118:4-118:52: Tainted HTML output — possible XSS
TaintedHtml@125:4-125:58: Tainted HTML output — possible XSS
TaintedHtml@126:4-126:58: Tainted HTML output — possible XSS
TaintedHtml@127:4-127:35: Tainted HTML output — possible XSS
TaintedHtml@128:4-128:52: Tainted HTML output — possible XSS
TaintedHtml@133:4-133:49: Tainted HTML output — possible XSS
TaintedHtml@135:4-135:59: Tainted HTML output — possible XSS
