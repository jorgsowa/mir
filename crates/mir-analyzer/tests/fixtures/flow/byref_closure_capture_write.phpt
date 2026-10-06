===description===
A closure writing a by-reference capture (`use (&$x)`) updates the parent
variable: the write is merged back after the closure literal, so later reads
see the written type instead of the pre-closure one.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function keyWrite(): void {
    $arr = [];
    $fill = function () use (&$arr) {
        $arr['k'] = 1;
    };
    $fill();
    echo $arr['k'];
}

function scalarWrite(): void {
    $n = 0;
    $inc = function () use (&$n) {
        $n = 'done';
    };
    $inc();
    /** @mir-check $n is int|string */
    $_ = $n;
}

function appendWrite(): void {
    $items = [];
    $add = function (int $i) use (&$items) {
        $items[] = $i;
    };
    $add(1);
    /** @mir-check $items is list<int> */
    $_ = $items;
}

function byValueUnaffected(): void {
    $n = 0;
    $f = function () use ($n) {
        $n = 'x';
    };
    $f();
    /** @mir-check $n is 0 */
    $_ = $n;
}

function conditionalWrite(bool $b): void {
    $r = null;
    $f = function () use (&$r) {
        if ($b ?? false) {
            $r = 5;
        }
    };
    $f();
    /** @mir-check $r is mixed */
    $_ = $r;
}

function arrowFnIsByValue(): void {
    $n = 0;
    $f = fn() => $n = 'x';
    $f();
    /** @mir-check $n is 0 */
    $_ = $n;
}
