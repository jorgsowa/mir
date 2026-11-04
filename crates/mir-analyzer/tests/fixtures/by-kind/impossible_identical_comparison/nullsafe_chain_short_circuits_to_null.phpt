===description===
A `->` after `?->` is skipped when the chain hits null, so the result can be null
===file===
<?php
interface Meta {
    public function attrs(): array;
    public function owner(): Meta;
    public string $name { get; }
}

function f(?Meta $m): int {
    $attrs = $m?->owner()->attrs();
    $name = $m?->owner()->name;
    if (null === $attrs || null === $name) {
        return 0;
    }
    return \count($attrs);
}
