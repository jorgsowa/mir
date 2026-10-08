===description===
`hrtime(true)` returns int; the array form never includes false, so indexing it is clean.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Progress {
    private ?int $firstAt = null;
    public function tick(): int {
        $this->firstAt ??= hrtime(true);
        return $this->firstAt;
    }
}
$a = hrtime(true);
/** @mir-check $a is int */
$b = hrtime(false);
/** @mir-check $b is array{0: int, 1: int} */
$c = hrtime();
/** @mir-check $c is array{0: int, 1: int} */
function g(bool $flag): void {
    $d = hrtime($flag);
    /** @mir-check $d is array{0: int, 1: int}|int */
}
function h(): int {
    $t = hrtime();
    $first = $t[0];
    $pair = hrtime(false);
    return $first + $pair[1];
}
