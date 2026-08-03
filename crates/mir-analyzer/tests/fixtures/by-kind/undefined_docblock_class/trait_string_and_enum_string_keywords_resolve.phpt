===description===
M23: `trait-string` and `enum-string` are recognized docblock type
keywords (parallel to the already-working `class-string`/
`interface-string`), not undefined class names.
===config===
suppress=UnusedParam
===file===
<?php
trait HasName {}
enum Status { case Active; }

class Holder {
    /** @var trait-string */
    private string $traitName;

    /**
     * @param trait-string $traitName
     */
    public function __construct(string $traitName)
    {
        $this->traitName = $traitName;
    }

    /**
     * @return trait-string
     */
    public function traitName(): string
    {
        return $this->traitName;
    }
}

/** @param enum-string $e */
function acceptsEnumString(string $e): void {}
===expect===
