===description===
An unresolved docblock class returned from a static method does not cascade at function or constructor calls.
===file===
<?php
namespace App;

class Factory {
    /** @return Gone */
    public static function make() { return 1; }
//                         ^^^^ UndefinedDocblockClass: Docblock type 'App\Gone' does not exist
}

class Needs {
    public function __construct(public int $n) {}
}

function build(): Needs {
    echo strlen(Factory::make());
    return new Needs(Factory::make());
}
