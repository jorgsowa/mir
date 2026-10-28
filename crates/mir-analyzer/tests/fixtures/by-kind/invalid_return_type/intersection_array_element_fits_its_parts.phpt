===description===
An array of `A&B` elements is an array of `A` (and of `B`)
===file===
<?php
interface Focusable {}
abstract class Widget {}

final class Focus {
    /** @var array<int, Focusable&Widget> */
    private array $items = [];

    /** @return Focusable[] */
    public function all(): array {
        return $this->items;
    }

    /** @return list<Widget> */
    public function widgets(): array {
        return array_values($this->items);
    }

    /** @return list<string> */
    public function names(): array {
        return array_values($this->items);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'list<Focusable&Widget>' is not compatible with declared 'list<string>'
    }
}
