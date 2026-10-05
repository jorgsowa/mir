===description===
An `@inheritDoc` override keeps its own native param type when the parent's docblock param type does not mention a template.
===file===
<?php
interface Source
{
    /** @param non-empty-string $key */
    public function read(string $key): int;
}

final class Impl implements Source
{
    /** @inheritDoc */
    public function read(string $key): int
    {
        return strlen($key);
    }
}

function call(Impl $impl, string $key): void
{
    echo $impl->read($key);
}
===expect===
