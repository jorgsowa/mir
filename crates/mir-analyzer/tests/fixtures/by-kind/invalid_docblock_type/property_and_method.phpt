===description===
`@property` and magic `@method` type positions are checked the same way.
===file===
<?php
class Bag {
    /**
     * @property \int $count
     * @method \string name(int $id)
     */
    public array $items = [];
}
===expect===
InvalidDocblockType@4:17-4:21: Invalid docblock type: @property backslash-qualified non-class type '\int' is not a fully qualified name
InvalidDocblockType@5:15-5:22: Invalid docblock type: @method backslash-qualified non-class type '\string' is not a fully qualified name
