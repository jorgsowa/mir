===description===
`str_replace`, `str_ireplace` and `preg_replace` copy only the replacement and the
subject into the result, never the search term or pattern. `str_word_count`
without a format returns a count.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <PossiblyNullArgument errorLevel="suppress"/>
    <PossiblyInvalidArrayAccess errorLevel="suppress"/>
    <TaintedInput errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function searchTermDoesNotCarry(): void {
    echo str_replace($_GET['a'], 'x', 'abc');
    echo str_ireplace($_GET['a'], 'x', 'abc');
    echo str_replace([$_GET['a']], ['x'], 'abc');
    echo preg_replace($_GET['a'], 'x', 'abc');
}

function replacementAndSubjectCarry(): void {
    echo str_replace('a', $_GET['a'], 'abc');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo str_replace('a', 'b', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo str_ireplace('a', $_GET['a'], 'abc');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo str_ireplace('a', 'b', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo str_replace('a', ['x' => $_GET['a']], 'abc');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/a/', $_GET['a'], 'abc');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo preg_replace('/a/', 'b', $_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function namedAndSpreadArgumentsStayConservative(): void {
    echo str_replace(search: $_GET['a'], replace: 'x', subject: 'abc');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo str_replace(...[$_GET['a'], 'x', 'abc']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function wordCount(): void {
    echo str_word_count($_GET['a']);
    $n = str_word_count($_GET['a']);
    echo $n;
}

function wordCountWithFormatReturnsWords(): void {
    echo str_word_count($_GET['a'], 1)[0];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo str_word_count($_GET['a'], 2)[0];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function throughVariable(): void {
    $s = str_replace($_GET['a'], 'x', 'abc');
    echo $s;
    $t = str_replace('a', $_GET['a'], 'abc');
    echo $t;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
TaintedHtml@20:4-20:48: Tainted HTML output — possible XSS
TaintedHtml@22:4-22:46: Tainted HTML output — possible XSS
TaintedHtml@27:4-27:71: Tainted HTML output — possible XSS
TaintedHtml@29:4-29:50: Tainted HTML output — possible XSS
