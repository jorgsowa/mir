===description===
P28 sibling: the same docblock-shadowed-builtin storage bug hits a class
method's `@param`/`@return` too (collected via `collector/class.rs`'s method
path, consumed via `method_chain_signature` in body analysis).
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

final class Factory
{
    /**
     * @param Generator $g
     * @return Generator
     */
    public function useIt(Generator $g): Generator
    {
        $g->build();
        return $g;
    }
}
===expect===
