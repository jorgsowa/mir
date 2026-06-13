===description===
generic class without constructor yields bare type and does not panic
===file:bag.php===
<?php
/**
 * @template T of \Stringable
 */
class Bag {
    /**
     * @param T $item
     * @suppress UnusedParam
     */
    public function add($item): void {}
}
===file:app.php===
<?php
function app(): void {
    $b = new Bag();
    echo get_class($b);
}
===expect===
