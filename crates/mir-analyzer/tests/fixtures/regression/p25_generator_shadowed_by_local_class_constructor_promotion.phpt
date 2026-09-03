===description===
P25: a same-namespace user class named identically to the built-in `Generator`
iterator class must resolve a constructor-promoted property's native type hint to
the local FQCN, not the global builtin. `resolve_type_name`'s docblock-only
builtin-leniency shortcut (`is_global_builtin_docblock_class`) was previously applied
to native type hints too, so the promoted property's `class` field came back as bare
"Generator" instead of `App\Generator`, and the member-lookup call sites that compare
against the bare builtin name flagged every method on the local class as undefined.
Found in phpunit-phpunit (`PHPUnit\Runner\Baseline\Generator` vs.
`Subscriber::generator(): Generator`).
===config===
suppress=UnusedParam
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

final class Consumer
{
    public function __construct(private Generator $generator) {}
    public function run(): string { return $this->generator->build(); }
}
===expect===
