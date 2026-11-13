===description===
A `@var Generator` on an untyped property names the same-namespace `Generator`
declared in another file, not the built-in one. Without a local class the bare
name is still the built-in `\Generator`.
===file:Generator.php===
<?php
namespace Amp\Gen;

interface Generator {
    public function generate(): string;
}
===file:Holder.php===
<?php
namespace Amp\Gen;

final class Holder {
    /** @var Generator */
    private $g;

    public function __construct(Generator $g) {
        $this->g = $g;
    }

    public function run(): string {
        return $this->g->generate();
    }
}
===file:Builtin.php===
<?php
namespace Other;

final class Runner {
    /** @var Generator */
    private $g;

    public function __construct(\Generator $g) {
        $this->g = $g;
    }

    public function first(): mixed {
        return $this->g->current();
    }
}
