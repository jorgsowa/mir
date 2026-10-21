===description===
Arithmetic, shifts, comparisons and boolean operators yield a number or a bool,
so their result carries no payload. `+` on two non-scalar operands may still be
an array union, and `.` copies the operands' bytes.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function plus(): void {
    $n = $_GET['a'] + 1;
    echo $n;
}

function minusMulDivModPow(): void {
    echo $_GET['a'] - 1;
    echo 2 * $_GET['a'];
    echo $_GET['a'] / 2;
    echo $_GET['a'] % 2;
    echo $_GET['a'] ** 2;
    echo -$_GET['a'];
    echo +$_GET['a'];
}

function shiftsAndComparisons(): void {
    echo $_GET['a'] << 1;
    echo $_GET['a'] >> 1;
    echo $_GET['a'] <=> 'x';
    echo $_GET['a'] == 'x';
    echo $_GET['a'] === 'x';
    echo $_GET['a'] < 5;
    echo !$_GET['a'];
    echo $_GET['a'] && true;
    echo $_GET['a'] || false;
    echo $_GET['a'] xor true;
    echo $_GET['a'] instanceof Countable;
}

function chained(): void {
    $n = ($_GET['a'] + 1) * 2;
    $m = $n - $_GET['b'];
    echo "total: $m";
}

function variableOperand(int $limit): void {
    echo $_GET['a'] + $limit;
}

function compound(): void {
    $total = 0;
    $total += $_GET['a'];
    $total -= $_GET['b'];
    $total *= $_GET['c'];
    echo $total;
}

function compoundOnTaintedLeft(): void {
    $x = $_GET['a'];
    $x -= 1;
    echo $x;
}

function arrayUnionKeepsTaint(): void {
    $merged = $_GET + ['page' => '1'];
    echo implode(',', $merged);
}

function mixedOperandsKeepTaint(mixed $defaults): void {
    $merged = $_GET['a'] + $defaults;
    echo $merged;
}

function compoundArrayUnionKeepsTaint(array $defaults): void {
    $defaults += $_GET;
    echo implode(',', $defaults);
}

function concatKeepsTaint(): void {
    echo $_GET['a'] . 1;
    echo 'n' . ($_GET['a'] + 1) . $_GET['b'];
}

function bitwiseWithScalar(): void {
    echo $_GET['a'] & 0xff;
}
===expect===
TaintedHtml@57:4-57:31: Tainted HTML output — possible XSS
TaintedHtml@62:4-62:17: Tainted HTML output — possible XSS
TaintedHtml@67:4-67:33: Tainted HTML output — possible XSS
TaintedHtml@71:4-71:24: Tainted HTML output — possible XSS
TaintedHtml@72:4-72:45: Tainted HTML output — possible XSS
