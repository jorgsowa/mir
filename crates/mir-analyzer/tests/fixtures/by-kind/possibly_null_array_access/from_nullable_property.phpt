===description===
Accessing an element of a class property declared as nullable array fires PossiblyNullArrayAccess
===file===
<?php
class Container {
    /** @var array<int>|null */
    public ?array $items;

    public function first(): void {
        echo $this->items[0];
//           ^^^^^^^^^^^^^^^ PossiblyNullArrayAccess: Cannot access array on possibly null value
    }
}
