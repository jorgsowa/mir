===description===
`list<int>` is an `array<array-key,int>`.
===config===
php_version=8.4
===file===
<?php
class Holder {
    /** @var array<array-key, int> */
    private array $data;
    /** @param list<int> $items */
    public function __construct(array $items) {
        $this->data = $items;
    }
}
===expect===
