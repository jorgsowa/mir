===description===
M26: `instanceof` against a `callable`-typed value narrows like TObject/
TMixed instead of always being considered impossible — `callable`
legitimately includes an invokable object of any class. A `Closure`-typed
value (a real, final PHP class) checked against an unrelated class is
still genuinely impossible and must keep flagging.
===config===
suppress=UnusedParam
===file===
<?php
final class Handler {
    public function __invoke(): void {}

    /** @param list<callable> $handlers */
    public function check(array $handlers): void {
        foreach ($handlers as $handler) {
            if ($handler instanceof self) {
                echo "match\n";
            }
        }
    }
}

function stillImpossible(\Closure $c): void {
    if ($c instanceof Handler) {
//      ^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        echo "unreachable\n";
    }
}
===expect===
