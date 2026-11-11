===description===
`@phan-read-only` is read as `@readonly`: the property may be read anywhere, but assigning it outside the constructor is `ReadonlyPropertyAssignment`.
===file===
<?php
final class Bag
{
    /** @phan-read-only */
    public int $id;

    public function __construct()
    {
        $this->id = 1;
    }
}

$bag = new Bag();
echo $bag->id;
$bag->id = 2;
//<^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Bag::$id outside of constructor
