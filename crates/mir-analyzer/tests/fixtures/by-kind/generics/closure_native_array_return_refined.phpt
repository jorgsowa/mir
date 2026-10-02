===description===
A closure's bare native `array` return is refined by its body's array type, so a
template-bearing constructor infers the precise arg. Non-array and
non-subtype bodies keep the declared type.
===config===
suppress=UnusedVariable,UnusedParam,MissingParamType,MixedAssignment,MissingConstructor,UnnecessaryVarAnnotation
===file===
<?php
/** @template T */
final class Cached {
    /** @param \Closure(): T $f */
    public function __construct(private \Closure $f) {}
}

/** @return array<non-empty-string, mixed> */
function load(): array { return ['a' => 1]; }

/** @return list<int> */
function ids(): array { return [1]; }

final class Holder {
    /** @var Cached<array<non-empty-string, mixed>> */
    private Cached $c;

    public function arrow(): void {
        $this->c = new Cached(fn(): array => load());
    }

    public function closure(): void {
        $this->c = new Cached(function (): array { return load(); });
    }

    public function inferred(): void {
        $this->c = new Cached(fn() => load());
    }

    public function plain(): void {
        $e = new Cached(fn(): array => load());
        /** @mir-check $e is Cached<array<non-empty-string, mixed>> */
        $f = new Cached(function (): array { return load(); });
        /** @mir-check $f is Cached<array<non-empty-string, mixed>> */
        $a = new Cached(fn(): array => ids());
        /** @mir-check $a is Cached<list<int>> */
        $b = new Cached(fn(): array => [1, 2]);
        /** @mir-check $b is Cached<array> */
        $d = new Cached(fn(): iterable => ids());
        /** @mir-check $d is Cached<iterable<mixed, mixed>> */
    }
}
===expect===
