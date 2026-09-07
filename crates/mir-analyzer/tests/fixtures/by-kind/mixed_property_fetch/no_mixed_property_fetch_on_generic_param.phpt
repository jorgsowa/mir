===description===
Property access on a generic type parameter must not emit MixedPropertyFetch —
generic containers like Repository<T> routinely fetch properties from T values.
===config===
suppress=MissingPropertyType,MissingReturnType
===file===
<?php
/**
 * @template T
 */
class Repository {
    /** @var list<T> */
    private array $items = [];

    /** @param T $item */
    public function add($item): void {
        $this->items[] = $item;
    }

    public function processAll(): void {
        foreach ($this->items as $item) {
            // $item is T — must not fire MixedPropertyFetch
            $item->name;
            $item->process();
        }
    }
}
===expect===
