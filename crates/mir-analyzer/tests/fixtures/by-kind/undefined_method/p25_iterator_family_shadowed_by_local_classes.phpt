===description===
P25, remaining builtin-iterator-family names: `Iterator`, `IteratorAggregate`, and
`Traversable` all sit in `is_global_builtin_docblock_class`'s bare-name shortlist
alongside `Generator`, so a same-namespace class/interface reusing any of those
names must resolve the same way — via a param type hint (`Iterator`), a return type
hint (`Traversable`), and an implemented-interface type hint (`IteratorAggregate`).
===config===
suppress=UnusedParam
===file===
<?php

namespace App;

interface Iterator
{
    public function step(): string;
}

interface Traversable
{
    public function traverse(): string;
}

interface IteratorAggregate
{
    public function label(): string;
}

final class Walker
{
    public function walk(Iterator $it): string
    {
        return $it->step();
    }

    public function cross(): Traversable
    {
        return new class implements Traversable {
            public function traverse(): string { return 'x'; }
        };
    }

    public function useCross(): string
    {
        return $this->cross()->traverse();
    }

    public function describe(IteratorAggregate $agg): string
    {
        return $agg->label();
    }
}
===expect===
