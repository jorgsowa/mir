===description===
Property and magic method types reject backslash-qualified keywords.
===file===
<?php
class Bag {
    /**
     * @property \int $count
//               ^^^^ InvalidDocblockType: Invalid docblock type: @property backslash-qualified non-class type '\int' is not a fully qualified name
     * @method \string name(int $id)
//             ^^^^^^^ InvalidDocblockType: Invalid docblock type: @method backslash-qualified non-class type '\string' is not a fully qualified name
     */
    public array $items = [];
}
