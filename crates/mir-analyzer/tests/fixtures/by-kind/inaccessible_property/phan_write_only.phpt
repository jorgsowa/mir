===description===
A `@phan-write-only` property can be written anywhere but read only inside its class: a read from outside, including a compound assignment, is `InaccessibleProperty`.
===file===
<?php
final class Bag
{
    /** @phan-write-only */
    public int $scratch = 0;

    public function useScratch(): int
    {
        return $this->scratch;
    }
}

$bag = new Bag();
$bag->scratch = 3;
$bag->scratch += 1;
//    ^^^^^^^ InaccessibleProperty: Cannot access property Bag::$scratch
echo $bag->scratch;
//         ^^^^^^^ InaccessibleProperty: Cannot access property Bag::$scratch
