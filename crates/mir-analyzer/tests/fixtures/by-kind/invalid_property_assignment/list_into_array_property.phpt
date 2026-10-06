===description===
`list<int>` is an `array<array-key,int>`.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
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
