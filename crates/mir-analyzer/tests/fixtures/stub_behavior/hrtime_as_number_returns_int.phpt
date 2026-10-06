===description===
`hrtime(true)` returns int, so `??=` and arithmetic keep int; no-arg form keeps the array shape.
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
/** @mir-check $b is array{0: int, 1: int}|false */
$c = hrtime();
/** @mir-check $c is array{0: int, 1: int}|false */
function g(bool $flag): void {
    $d = hrtime($flag);
    /** @mir-check $d is array{0: int, 1: int}|false|int */
}
