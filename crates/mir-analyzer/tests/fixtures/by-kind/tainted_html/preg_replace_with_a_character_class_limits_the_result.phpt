===description===
`preg_replace()` with a single character class and a plain replacement leaves
only the characters the class does not match (plus the replacement's own). The
result is safe for the kinds those characters cannot break out of. Anything else
-- several patterns, a quantifier count, a limit, a back-reference, a tainted
replacement -- keeps the subject's taint.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <PossiblyInvalidOperand errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function keepsOnlyDigits(): void {
    echo preg_replace('/[^0-9]/', '', $_GET['a']);
    echo preg_replace('/\D/', '', $_GET['a']);
    echo preg_replace('/[^0-9]+/', '', $_GET['a']);
    echo preg_replace('~[^\d]*~', '', $_GET['a']);
    echo preg_replace('#[^0-9]#s', '', $_GET['a']);
}

function keepsWordCharacters(): void {
    echo preg_replace('/[^a-z0-9_]/i', '', $_GET['a']);
    echo preg_replace('/[^\w]/', '', $_GET['a']);
    echo preg_replace('/\W+/', '', $_GET['a']);
    echo preg_replace('/[^\w-]/u', '', $_GET['a']);
}

function replacementIsSafeText(): void {
    echo preg_replace('/[^0-9]/', '-', $_GET['a']);
    echo preg_replace('/[^0-9]/', '', $_GET['a']);
}

function replacementIsDangerous(): void {
    echo preg_replace('/[^0-9]/', '<b>', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function replacementCopiesTheMatch(): void {
    echo preg_replace('/[^0-9]/', '$0', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/[^0-9]/', '\0', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/[^0-9]/', '[$0]', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function replacementIsTainted(): void {
    echo preg_replace('/[^0-9]/', $_GET['b'], $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function removesOnlyTheListedCharacters(): void {
    echo preg_replace('/[<>]/', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/[<>"\'&]/', '', $_GET['a']);
    header('X-A: ' . preg_replace('/[\r\n]/', '', $_GET['a']));
    echo preg_replace('/\s/', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function headerKindNeedsNoLineBreaks(): void {
    header('X-A: ' . preg_replace('/[^a-z0-9 ]/', '', $_GET['a']));
    header('X-A: ' . preg_replace('/[^\s\d]/', '', $_GET['a']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
    header('X-A: ' . preg_replace('/[^0-9]/', "\n", $_GET['a']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
}

function sqlKind(mysqli $db): void {
    mysqli_query($db, 'SELECT * FROM t WHERE id = ' . preg_replace('/[^0-9]/', '', $_GET['id']));
    mysqli_query($db, "SELECT * FROM t WHERE n = '" . preg_replace('/[^a-z]/', '', $_GET['n']) . "'");
    mysqli_query($db, "SELECT * FROM t WHERE n = '" . preg_replace('/[^a-z ]/', '', $_GET['n']) . "'");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}

function notASingleClass(): void {
    echo preg_replace('/[^0-9]{2}/', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/[^0-9]|x/', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/^[^0-9]/', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/a[^0-9]/', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/([^0-9])/', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/./', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace(['/[^0-9]/', '/x/'], '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/[^0-9]/', ['x'], $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function modifiersThatChangeMatching(): void {
    echo preg_replace('/ [^0-9]/x', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/[^0-9]/A', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/[^0-9]/e', '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function limitedReplacementLeavesTheRest(): void {
    echo preg_replace('/[^0-9]/', '', $_GET['a'], 3);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/[^0-9]/', '', $_GET['a'], -1);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function patternNotALiteral(string $pattern): void {
    echo preg_replace($pattern, '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/[^0-9]/' . $pattern, '', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function arraySubject(): void {
    foreach (preg_replace('/[^0-9]/', '', $_GET['l']) as $v) {
        echo $v;
    }
}

function unicodeShorthandWidensTheResult(): void {
    $callable = preg_replace('/[^\w]/u', '', $_GET['f']);
    $callable();
//  ^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
}

function asciiLettersAreStillAFunctionName(): void {
    $callable = preg_replace('/[^a-z]/', '', $_GET['f']);
    $callable();
//  ^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    $digits = preg_replace('/[^0-9]/', '', $_GET['f']);
    $digits();
}

function inAWrapper(): void {
    echo digitsOf($_GET['a']);
    echo wordsOf($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function digitsOf(string $s): string {
    return preg_replace('/\D/', '', $s);
}

function wordsOf(string $s): string {
    return preg_replace('/[\s]/', '', $s);
}

function inAnArrowFunction(): void {
    echo implode(',', array_map(fn($v) => preg_replace('/\D/', '', $v), $_GET['l']));
    echo implode(',', array_map(fn($v) => preg_replace('/\s/', '', $v), $_GET['l']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
