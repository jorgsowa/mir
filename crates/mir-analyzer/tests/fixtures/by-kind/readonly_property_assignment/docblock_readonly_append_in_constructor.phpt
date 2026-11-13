===description===
A `@psalm-readonly` array may be appended to in its own constructor, but not elsewhere.
===file===
<?php
final class P {
    /**
     * @var list<string>
     * @psalm-readonly
     */
    private array $prefixes = ['www.'];

    /** @param list<string> $protos */
    public function __construct(array $protos) {
        foreach ($protos as $p) {
            $this->prefixes[] = $p . '://';
        }
    }

    public function add(string $p): void {
        $this->prefixes[] = $p;
//      ^^^^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property P::$prefixes outside of constructor
    }
}
