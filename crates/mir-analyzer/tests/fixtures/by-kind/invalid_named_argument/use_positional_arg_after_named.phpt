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
===expect===
ParseError@10:21-10:22: Parse error: cannot use positional argument after named argument
