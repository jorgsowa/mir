===description===
`@psalm-type` / `@phpstan-type` accept `Name = Expr` and `Name Expr`, a generic name, and `=` inside the expression.
===file===
<?php
namespace Lib;

/**
 * @psalm-type Row array{id: int, name: string}
 * @psalm-type Pair = array{0: int, 1: string}
 * @psalm-type Tag='a=b'|'c'
 * @phpstan-type Loose array{id: int}
 * @psalm-type ListOf<T> array<int, T>
 */
final class Holder {
    /** @return Row */
    public function row(): array {
        return ['id' => 1, 'name' => 'x'];
    }

    /** @return Pair */
    public function pair(): array {
        return [1, 'x'];
    }

    /** @return Tag */
    public function tag(): string {
        return 'c';
    }

    /** @return Loose */
    public function loose(): array {
        return ['id' => 1];
    }

    /** @return ListOf */
    public function many(): array {
        return [];
    }
}

function check(Holder $h): void {
    /** @mir-check $h->row() is array{id: int, name: string} */
    $h->row();
    /** @mir-check $h->pair() is array{0: int, 1: string} */
    $h->pair();
    /** @mir-check $h->tag() is "a=b"|"c" */
    $h->tag();
    /** @mir-check $h->loose() is array{id: int} */
    $h->loose();
}
===expect===
