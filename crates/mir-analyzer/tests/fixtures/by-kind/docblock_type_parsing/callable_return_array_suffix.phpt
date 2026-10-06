===description===
A `[]` after a callable's `: ret` belongs to the return type, not to the
whole callable; a parenthesised callable followed by `[]` stays an array of
callables.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function check_closure_return_array($x) {
    /**
     * @var Closure(): string[] $x
     * @mir-check $x is Closure(): array<array-key, string>
     */
    var_dump($x);
}

function check_callable_return_array($x) {
    /**
     * @var callable(int): int[] $x
     * @mir-check $x is callable(int): array<array-key, int>
     */
    var_dump($x);
}

function check_return_nested_array($x) {
    /**
     * @var Closure(): string[][] $x
     * @mir-check $x is Closure(): array<array-key, array<array-key, string>>
     */
    var_dump($x);
}

function check_nullable_closure_returning_array($x) {
    /**
     * @var Closure(): string[]|null $x
     * @mir-check $x is Closure(): array<array-key, string>|null
     */
    var_dump($x);
}

function check_parenthesised_callable_array($x) {
    /**
     * @var (Closure(): string)[] $x
     * @mir-check $x is array<array-key, Closure(): string>
     */
    var_dump($x);
}

function check_generic_arg_closure_returning_array($x) {
    /**
     * @var list<Closure(): string[]> $x
     * @mir-check $x is list<Closure(): array<array-key, string>>
     */
    var_dump($x);
}

class Provider {
    /** @param Closure(): string[] $provider */
    public function __construct(private Closure $provider) {}

    /** @return Closure(): int[] */
    public function make(): Closure {
        return static fn(): array => [1];
    }
}

new Provider(static fn(): array => ['a']);

/** @param callable(): int[] $c */
function takes_callable(callable $c): void { $c(); }
takes_callable(static fn(): array => [1]);
