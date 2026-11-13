===description===
A class template used in an `object{…}` shape property is the template, so its
`int|string` bound makes it a valid array offset.
===file===
<?php
/** @template T of int|string */
final class Q {
    /** @var array<int, object{key: T}> */
    private array $data = [];
    /** @var array<T, int> */
    private array $pointers = [];

    public function rm(int $node): void {
        unset($this->pointers[$this->data[$node]->key]);
    }
}
