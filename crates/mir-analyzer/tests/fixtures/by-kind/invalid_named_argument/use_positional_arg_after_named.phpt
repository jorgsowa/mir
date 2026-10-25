===description===
Use positional arg after named
===file===
<?php
final class Person
{
    public function __construct(
        public string $name,
        public int $age,
    ) { }
}

new Person(name: "", 0);
//                   ^ CompileError: Cannot use positional argument after named argument [positional-after-named-argument]
