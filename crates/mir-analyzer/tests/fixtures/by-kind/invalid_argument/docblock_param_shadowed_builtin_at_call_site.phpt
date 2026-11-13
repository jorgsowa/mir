===description===
A `@param Generator` in a namespace with its own `Generator` class is that class
when the method or function is called from another file, so passing it is
accepted and the return type is the local class.
===file:Generator.php===
<?php
namespace Faker;

class Generator {
    public function name(): string { return 'n'; }
}
===file:Factory.php===
<?php
namespace Faker;

final class Factory {
    /**
     * @param Generator $g
     * @return Generator
     */
    public function wrap($g) { return $g; }
}

/** @param Generator $g */
function seed($g): void { echo $g->name(); }
===file:Use.php===
<?php
use Faker\Factory;
use Faker\Generator;

function run(Factory $f, Generator $g): string {
    \Faker\seed($g);
    return $f->wrap($g)->name();
}
