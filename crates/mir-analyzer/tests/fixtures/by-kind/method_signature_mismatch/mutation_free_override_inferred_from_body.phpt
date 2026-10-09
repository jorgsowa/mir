===description===
An override that does not repeat @mutation-free / @external-mutation-free is accepted
when its body provably writes no property (reads, locals, pure builtins, clean
sibling methods); one that mutates, or calls something unknown, is still reported.
===file===
<?php
interface Ctx {
    /** @psalm-mutation-free */
    public function getId(): string;
    /** @psalm-mutation-free */
    public function label(): string;
    /** @psalm-mutation-free */
    public function size(): int;
    /** @psalm-mutation-free */
    public function touched(): int;
    /** @psalm-mutation-free */
    public function unknownCall(): int;
    /** @psalm-external-mutation-free */
    public function peek(): string;
}

final class Reader implements Ctx {
    private int $hits = 0;
    /** @param list<string> $parts */
    public function __construct(private string $id, private array $parts = []) {}

    public function getId(): string { return $this->id; }

    public function label(): string { return strtoupper($this->getId()); }

    public function size(): int {
        $total = 0;
        foreach ($this->parts as $part) {
            $total += strlen($part);
        }
        return $total;
    }

    public function touched(): int {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Reader::touched() signature mismatch: Ctx::touched() is declared @mutation-free and must be re-declared @mutation-free when overridden
        $this->hits++;
        return $this->hits;
    }

    public function unknownCall(): int {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Reader::unknowncall() signature mismatch: Ctx::unknowncall() is declared @mutation-free and must be re-declared @mutation-free when overridden
        return $this->mutate();
    }

    public function peek(): string { return $this->id; }

    private function mutate(): int { $this->hits++; return $this->hits; }
}
