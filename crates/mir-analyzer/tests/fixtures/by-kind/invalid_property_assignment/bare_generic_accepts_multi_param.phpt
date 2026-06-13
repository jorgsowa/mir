===description===
bare generic property accepts multi-parameter type
===file===
<?php
/** @template K
 *  @template V
 */
class Map {}

class Registry {
    private Map $mapping;

    public function register(): void {
        /** @var Map<string, int> $m */
        $m = new Map();
        $this->mapping = $m;
    }
}
===expect===
MissingConstructor@7:0-7:16: Class Registry has uninitialized properties but no constructor
