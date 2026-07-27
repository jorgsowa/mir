===description===
A static call's own is_in_immutable_method check only scoped itself to
self::/parent:: (mutating $this directly) and is_in_external_mutation_free_
method was never referenced at all in this file — a genuinely static call
(Foo::bar($this)) or a variable-class-string call ($param::method($this))
passing an object argument reachable from $this/a parameter went
completely unflagged, unlike new X(...) and free-function calls.
===config===
suppress=MissingConstructor,MixedArgument
===file===
<?php
class Box {
    public bool $touched = false;
}

class Logger {
    public static function record(Box $o): void {
        $o->touched = true;
    }
}

/** @psalm-immutable */
class Holder {
    public Box $box;

    public function corruptThis(): void {
        Logger::record($this->box);
    }
}

class Wrapper {
    /** @psalm-external-mutation-free */
    public function corruptParam(Box $box): void {
        Logger::record($box);
    }

    /** @psalm-external-mutation-free */
    public function safeValue(): void {
        Logger::record(new Box());
    }

    /** @psalm-external-mutation-free */
    public function corruptViaVariableClassString(Box $box, string $loggerCls): void {
        /** @var class-string<Logger> $loggerCls */
        $loggerCls::record($box);
    }
}
===expect===
ImpureMethodCall@17:8-17:34: Calling impure method record() in a pure or immutable context
ImpureMethodCall@24:8-24:28: Calling impure method record() in a pure or immutable context
ImpureMethodCall@35:8-35:32: Calling impure method record() in a pure or immutable context
