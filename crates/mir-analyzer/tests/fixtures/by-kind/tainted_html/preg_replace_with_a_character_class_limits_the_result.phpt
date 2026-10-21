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
}

function replacementCopiesTheMatch(): void {
    echo preg_replace('/[^0-9]/', '$0', $_GET['a']);
    echo preg_replace('/[^0-9]/', '\0', $_GET['a']);
    echo preg_replace('/[^0-9]/', '[$0]', $_GET['a']);
}

function replacementIsTainted(): void {
    echo preg_replace('/[^0-9]/', $_GET['b'], $_GET['a']);
}

function removesOnlyTheListedCharacters(): void {
    echo preg_replace('/[<>]/', '', $_GET['a']);
    echo preg_replace('/[<>"\'&]/', '', $_GET['a']);
    header('X-A: ' . preg_replace('/[\r\n]/', '', $_GET['a']));
    echo preg_replace('/\s/', '', $_GET['a']);
}

function headerKindNeedsNoLineBreaks(): void {
    header('X-A: ' . preg_replace('/[^a-z0-9 ]/', '', $_GET['a']));
    header('X-A: ' . preg_replace('/[^\s\d]/', '', $_GET['a']));
    header('X-A: ' . preg_replace('/[^0-9]/', "\n", $_GET['a']));
}

function sqlKind(mysqli $db): void {
    mysqli_query($db, 'SELECT * FROM t WHERE id = ' . preg_replace('/[^0-9]/', '', $_GET['id']));
    mysqli_query($db, "SELECT * FROM t WHERE n = '" . preg_replace('/[^a-z]/', '', $_GET['n']) . "'");
    mysqli_query($db, "SELECT * FROM t WHERE n = '" . preg_replace('/[^a-z ]/', '', $_GET['n']) . "'");
}

function notASingleClass(): void {
    echo preg_replace('/[^0-9]{2}/', '', $_GET['a']);
    echo preg_replace('/[^0-9]|x/', '', $_GET['a']);
    echo preg_replace('/^[^0-9]/', '', $_GET['a']);
    echo preg_replace('/a[^0-9]/', '', $_GET['a']);
    echo preg_replace('/([^0-9])/', '', $_GET['a']);
    echo preg_replace('/./', '', $_GET['a']);
    echo preg_replace(['/[^0-9]/', '/x/'], '', $_GET['a']);
    echo preg_replace('/[^0-9]/', ['x'], $_GET['a']);
}

function modifiersThatChangeMatching(): void {
    echo preg_replace('/ [^0-9]/x', '', $_GET['a']);
    echo preg_replace('/[^0-9]/A', '', $_GET['a']);
    echo preg_replace('/[^0-9]/e', '', $_GET['a']);
}

function limitedReplacementLeavesTheRest(): void {
    echo preg_replace('/[^0-9]/', '', $_GET['a'], 3);
    echo preg_replace('/[^0-9]/', '', $_GET['a'], -1);
}

function patternNotALiteral(string $pattern): void {
    echo preg_replace($pattern, '', $_GET['a']);
    echo preg_replace('/[^0-9]/' . $pattern, '', $_GET['a']);
}

function arraySubject(): void {
    foreach (preg_replace('/[^0-9]/', '', $_GET['l']) as $v) {
        echo $v;
    }
}

function unicodeShorthandWidensTheResult(): void {
    $callable = preg_replace('/[^\w]/u', '', $_GET['f']);
    $callable();
}

function asciiLettersAreStillAFunctionName(): void {
    $callable = preg_replace('/[^a-z]/', '', $_GET['f']);
    $callable();
    $digits = preg_replace('/[^0-9]/', '', $_GET['f']);
    $digits();
}

function inAWrapper(): void {
    echo digitsOf($_GET['a']);
    echo wordsOf($_GET['a']);
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
}
===expect===
TaintedHtml@23:4-23:53: Tainted HTML output — possible XSS
TaintedHtml@27:4-27:52: Tainted HTML output — possible XSS
TaintedHtml@28:4-28:52: Tainted HTML output — possible XSS
TaintedHtml@29:4-29:54: Tainted HTML output — possible XSS
TaintedHtml@33:4-33:58: Tainted HTML output — possible XSS
TaintedHtml@37:4-37:48: Tainted HTML output — possible XSS
TaintedHtml@40:4-40:46: Tainted HTML output — possible XSS
TaintedHeader@45:4-45:63: Tainted HTTP header — possible header injection or open redirect
TaintedHeader@46:4-46:64: Tainted HTTP header — possible header injection or open redirect
TaintedSql@52:4-52:102: Tainted SQL query — possible SQL injection
TaintedHtml@56:4-56:53: Tainted HTML output — possible XSS
TaintedHtml@57:4-57:52: Tainted HTML output — possible XSS
TaintedHtml@58:4-58:51: Tainted HTML output — possible XSS
TaintedHtml@59:4-59:51: Tainted HTML output — possible XSS
TaintedHtml@60:4-60:52: Tainted HTML output — possible XSS
TaintedHtml@61:4-61:45: Tainted HTML output — possible XSS
TaintedHtml@62:4-62:59: Tainted HTML output — possible XSS
TaintedHtml@63:4-63:53: Tainted HTML output — possible XSS
TaintedHtml@67:4-67:52: Tainted HTML output — possible XSS
TaintedHtml@68:4-68:51: Tainted HTML output — possible XSS
TaintedHtml@69:4-69:51: Tainted HTML output — possible XSS
TaintedHtml@73:4-73:53: Tainted HTML output — possible XSS
TaintedHtml@74:4-74:54: Tainted HTML output — possible XSS
TaintedHtml@78:4-78:48: Tainted HTML output — possible XSS
TaintedHtml@79:4-79:61: Tainted HTML output — possible XSS
TaintedCallable@90:4-90:15: Tainted callable — user input chooses the code that runs
TaintedCallable@95:4-95:15: Tainted callable — user input chooses the code that runs
TaintedHtml@102:4-102:29: Tainted HTML output — possible XSS
TaintedHtml@115:4-115:85: Tainted HTML output — possible XSS
